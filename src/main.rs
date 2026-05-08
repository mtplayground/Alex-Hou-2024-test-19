#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use alex_hou_2024_test_19::{
        app::{shell, App},
        config::AppConfig,
    };
    use axum::Router;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tracing::info;

    let config = AppConfig::load()?;
    config.init_tracing()?;

    let addr = config.leptos_options.site_addr;
    let leptos_options = config.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    info!(
        site_addr = %addr,
        database_url_configured = !config.database_url.is_empty(),
        "starting leptos axum server"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
