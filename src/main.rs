use axum::serve;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use carteira_investimentos::routes::create_router;
use init_tracing_opentelemetry::TracingConfig;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = TracingConfig::development().init_subscriber()?;
    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await?;
    let router = create_router()
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());
    info!("Servidor escutando em {}", address);
    serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Falha ao escutar o sinal de interrupção");
        })
        .await?;
    info!("Servidor finalizado.");
    Ok(())
}
