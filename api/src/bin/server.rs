use anyhow::{Context, Result};

#[auto_context::auto_context]
#[tokio::main]
async fn main() -> Result<()> {
    let app = api::api::routes();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4096").await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
