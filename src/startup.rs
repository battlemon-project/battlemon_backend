use crate::config::{Config, DatabaseConfig};
use crate::graphql::QueryRoot;
use crate::routes;
use crate::routes::{setup_router, AppState, BattlemonSchema};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::routing::IntoMakeService;
use axum::{Router, Server};
use color_eyre::eyre::{Context, Result};
use hyper::server::conn::AddrIncoming;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use std::sync::Arc;

type HyperServer = Server<AddrIncoming, IntoMakeService<Router>>;

pub struct App {
    port: u16,
    server: HyperServer,
}

impl App {
    #[tracing::instrument(name = "Building application", skip_all)]
    pub async fn build(config: Config) -> Result<App> {
        tracing::info!("Connect to Postgres");
        let db_pool = get_db_pool(&config.db);
        let app_addr = format!("{}:{}", config.app.host, config.app.port);

        tracing::info!("Binding address - {app_addr} for app");
        let listener = TcpListener::bind(&app_addr).context("Failed to bind address for app")?;
        let port = listener.local_addr()?.port();

        tracing::info!("Compose GraphQL Schema");
        let graphql_schema = get_graphql_schema(&db_pool);
        let server =
            setup_server(listener, db_pool, graphql_schema).context("Failed to setup server")?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    #[tracing::instrument(name = "Starting application", skip_all)]
    pub async fn run_until_stopped(self) -> Result<()> {
        self.server.await.context("Failed to run server")
    }
}

#[tracing::instrument(name = "Setup server", skip_all)]
pub fn setup_server(
    listener: TcpListener,
    pool: PgPool,
    graphql_schema: BattlemonSchema,
) -> Result<HyperServer> {
    let state = AppState {
        db: pool,
        graphql: graphql_schema,
    };

    let router = setup_router(state);
    let server = axum::Server::from_tcp(listener)?.serve(router.into_make_service());

    Ok(server)
}

pub fn get_db_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}

pub fn get_graphql_schema(db_pool: &PgPool) -> BattlemonSchema {
    let db_pool = Arc::new(db_pool.clone());
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(db_pool)
        .finish()
}
