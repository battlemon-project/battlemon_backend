use crate::config::Config;
use crate::routes;
use axum::routing::IntoMakeService;
use axum::{Router, Server};
use color_eyre::eyre::{Context, Result};
use hyper::server::conn::AddrIncoming;
use std::net::TcpListener;

type HyperServer = Server<AddrIncoming, IntoMakeService<Router>>;

pub struct App {
    port: u16,
    server: HyperServer,
}

impl App {
    #[tracing::instrument(name = "Building application", skip(config))]
    pub async fn build(config: Config) -> Result<App> {
        let app_addr = format!("{}:{}", config.app.host, config.app.port);
        tracing::info!("Binding address - {app_addr} for app");
        let listener = TcpListener::bind(&app_addr).context("Failed to bind address for app")?;
        let port = listener.local_addr()?.port();
        let router = routes::get_router();
        let server = setup_server(listener, router).context("Failed to setup server")?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    #[tracing::instrument(name = "Starting application", skip(self))]
    pub async fn run_until_stopped(self) -> Result<()> {
        self.server.await.context("Failed to run server")
    }
}

#[tracing::instrument(name = "Setup server", skip(listener))]
pub fn setup_server(listener: TcpListener, router: Router) -> Result<HyperServer> {
    let server = axum::Server::from_tcp(listener)?.serve(router.into_make_service());

    Ok(server)
}
