use crate::AppState;
use crate::controllers::{auth_controller, users_controller};
use askama::Template;
use axum::extract::{Form, State};
use axum::response::{Html, Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use jwt_simple::prelude::*;
use serde::Deserialize;
use time::Duration as TimeDuration;
use validator::Validate;

#[derive(Template)]
#[template(path = "index.html.askama")]
pub struct IndexTemplate {
    pub logged_in: bool,
}

#[derive(Template)]
#[template(path = "register.html.askama")]
pub struct RegisterTemplate {
    pub error: String,
    pub logged_in: bool,
}

#[derive(Template)]
#[template(path = "login.html.askama")]
pub struct LoginTemplate {
    pub error: String,
    pub logged_in: bool,
}

#[derive(Template)]
#[template(path = "assets.html.askama")]
pub struct AssetsTemplate<'a> {
    pub assets: &'a [AssetTemplate],
    pub logged_in: bool,
}

pub struct AssetTemplate {
    pub id: String,
    pub name: String,
    pub ticker: String,
    pub category: String,
    pub quantity: String,
    pub average_price: String,
    pub total_invested: String,
    pub currency: String,
    pub last_acquisition_date: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct RegisterForm {
    #[validate(length(
        min = 3,
        max = 50,
        message = "O nome de usuário deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,

    #[validate(email(message = "O formato do e-mail informado é inválido"))]
    pub email: String,

    #[validate(length(
        min = 6,
        max = 100,
        message = "A senha deve ter entre 6 e 100 caracteres"
    ))]
    pub password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginForm {
    #[validate(length(
        min = 3,
        max = 50,
        message = "O nome de usuário deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,

    #[validate(length(
        min = 6,
        max = 100,
        message = "A senha deve ter entre 6 e 100 caracteres"
    ))]
    pub password: String,
}

async fn get_authenticated_user(jar: &CookieJar, jwt_secret: &str) -> Result<String, Redirect> {
    let access_token = jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| Redirect::to("/login"))?;
    let key = HS256Key::from_bytes(jwt_secret.as_bytes());
    let claims = key
        .verify_token::<NoCustomClaims>(&access_token, None)
        .map_err(|_| Redirect::to("/login"))?;
    claims.subject.ok_or_else(|| Redirect::to("/login"))
}

fn is_logged_in(jar: &CookieJar, jwt_secret: &str) -> bool {
    let Some(cookie) = jar.get("access_token") else {
        return false;
    };
    let key = HS256Key::from_bytes(jwt_secret.as_bytes());
    if let Ok(claims) = key.verify_token::<NoCustomClaims>(cookie.value(), None) {
        claims.subject.is_some()
    } else {
        false
    }
}

pub async fn index(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Html<String> {
    let logged_in = is_logged_in(&jar, &state.jwt_secret);
    Html(IndexTemplate { logged_in }.render().unwrap())
}

pub async fn register_form(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Html<String>, Redirect> {
    let logged_in = is_logged_in(&jar, &state.jwt_secret);
    if logged_in {
        return Err(Redirect::to("/assets"));
    }
    Ok(Html(
        RegisterTemplate {
            error: String::new(),
            logged_in,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn login_form(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Html<String>, Redirect> {
    let logged_in = is_logged_in(&jar, &state.jwt_secret);
    if logged_in {
        return Err(Redirect::to("/assets"));
    }
    Ok(Html(
        LoginTemplate {
            error: String::new(),
            logged_in,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, Html<String>> {
    let logged_in = is_logged_in(&jar, &state.jwt_secret);
    if let Err(errors) = form.validate() {
        return Err(RegisterTemplate {
            error: format!("Erro de validação: {}", errors),
            logged_in,
        }
        .render()
        .map(Html)
        .unwrap_or_else(|_| Html("<p>Erro ao processar registro</p>".to_string())));
    }

    match users_controller::create_user_internal(
        &state.pool,
        form.username,
        form.email,
        form.password,
    )
    .await
    {
        Ok(_) => Ok(Redirect::to("/login")),
        Err(error) => Err(RegisterTemplate {
            error: error.to_string(),
            logged_in,
        }
        .render()
        .map(Html)
        .unwrap_or_else(|_| Html("<p>Erro ao processar registro</p>".to_string()))),
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), Html<String>> {
    if let Err(_) = form.validate() {
        return Err(LoginTemplate {
            error: "Usuário ou senha inválidos".to_string(),
            logged_in: false,
        }
        .render()
        .map(Html)
        .unwrap_or_else(|_| Html("<p>Erro ao processar login</p>".to_string())));
    }

    let auth_request = auth_controller::AuthRequest {
        username: form.username,
        password: form.password,
    };

    match auth_controller::authenticate_user_internal(
        &state.pool,
        jar,
        auth_request,
        &state.jwt_secret,
    )
    .await
    {
        Ok((_status, jar, json)) => {
            let access_cookie = Cookie::build(("access_token", json.access_token.clone()))
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .max_age(TimeDuration::minutes(15))
                .build();
            Ok((jar.add(access_cookie), Redirect::to("/assets")))
        }
        Err(error) => {
            let login_error = match error {
                crate::errors::AppError::BadRequest(_) => error.to_string(),
                _ => "Usuário ou senha inválidos".to_string(),
            };
            Err(LoginTemplate {
                error: login_error,
                logged_in: false,
            }
            .render()
            .map(Html)
            .unwrap_or_else(|_| Html("<p>Erro ao processar login</p>".to_string())))
        }
    }
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    let clear_refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/api/v1/auth")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::ZERO)
        .build();
    let clear_access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::ZERO)
        .build();

    (
        jar.add(clear_refresh_cookie).add(clear_access_cookie),
        Redirect::to("/"),
    )
}

pub async fn assets(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Html<String>, Redirect> {
    let user_id = get_authenticated_user(&jar, &state.jwt_secret).await?;
    let user_id = uuid::Uuid::parse_str(&user_id).map_err(|_| Redirect::to("/login"))?;
    let assets = crate::models::assets_model::get_assets_by_user(&state.pool, user_id)
        .await
        .unwrap_or_default();
    let assets: Vec<AssetTemplate> = assets
        .into_iter()
        .map(|asset| {
            let quantity = asset.quantity_raw as f64 / 100.0;
            let average_price = asset.average_price_cents as f64 / 100.0;
            let total_invested = quantity * average_price;
            let formatted_date = asset
                .last_acquisition_date
                .map(|date| date.format("%d/%m/%Y").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            
            let symbol = match asset.currency.as_str() {
                "USD" => "$",
                "EUR" => "€",
                _ => "R$",
            };

            AssetTemplate {
                id: asset.id.to_string(),
                name: asset.name,
                ticker: asset.ticker.unwrap_or_else(|| "Sem Ticker".to_string()),
                category: asset.category,
                quantity: format!("{:.2}", quantity),
                average_price: format!("{} {:.2}", symbol, average_price),
                total_invested: format!("{} {:.2}", symbol, total_invested),
                currency: asset.currency,
                last_acquisition_date: formatted_date,
            }
        })
        .collect();
    Ok(Html(
        AssetsTemplate {
            assets: &assets,
            logged_in: true,
        }
        .render()
        .unwrap_or_else(|_| "<p>Erro ao renderizar ativos</p>".to_string()),
    ))
}
