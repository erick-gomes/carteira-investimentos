use crate::AppState;
use crate::controllers::{auth_controller, users_controller};
use askama::Template;
use axum::extract::{Form, State};
use axum::response::{Html, Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use jwt_simple::prelude::*;
use serde::Deserialize;
use time::Duration as TimeDuration;

#[derive(Template)]
#[template(path = "index.html.askama")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "register.html.askama")]
pub struct RegisterTemplate {
    pub error: String,
}

#[derive(Template)]
#[template(path = "login.html.askama")]
pub struct LoginTemplate {
    pub error: String,
}

#[derive(Template)]
#[template(path = "assets.html.askama")]
pub struct AssetsTemplate<'a> {
    pub assets: &'a [AssetTemplate],
}

pub struct AssetTemplate {
    pub name: String,
    pub ticker: String,
    pub quantity_raw: i64,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
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

pub async fn index() -> Html<String> {
    Html(IndexTemplate.render().unwrap())
}

pub async fn register_form() -> Html<String> {
    Html(
        RegisterTemplate {
            error: String::new(),
        }
        .render()
        .unwrap(),
    )
}

pub async fn login_form() -> Html<String> {
    Html(
        LoginTemplate {
            error: String::new(),
        }
        .render()
        .unwrap(),
    )
}

pub async fn register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, Html<String>> {
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
        Err(error) => Err(LoginTemplate {
            error: error.to_string(),
        }
        .render()
        .map(Html)
        .unwrap_or_else(|_| Html("<p>Erro ao processar login</p>".to_string()))),
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
        .map(|asset| AssetTemplate {
            name: asset.name,
            ticker: asset.ticker.unwrap_or_default(),
            quantity_raw: asset.quantity_raw,
        })
        .collect();
    Ok(Html(
        AssetsTemplate { assets: &assets }
            .render()
            .unwrap_or_else(|_| "<p>Erro ao renderizar ativos</p>".to_string()),
    ))
}
