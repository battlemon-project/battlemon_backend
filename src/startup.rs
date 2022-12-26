use crate::config::{Config, DatabaseConfig};
use crate::graphql::{MutationRoot, QueryRoot};
use crate::routes::{setup_router, AppState, BattlemonSchema};
use anyhow::{Context, Result};
use async_graphql::{EmptySubscription, Schema};
use axum::routing::IntoMakeService;
use axum::{Router, Server};
use hyper::server::conn::AddrIncoming;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;

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
        let graphql_schema = build_graphql_schema(&db_pool);
        tracing::info!("Export GraphQL Schema into SDL");
        save_sdl_to_file(&graphql_schema)
            .await
            .context("Failed to save GraphQL Schema to file")?;
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

pub fn build_graphql_schema(db_pool: &PgPool) -> BattlemonSchema {
    let db_pool = db_pool.clone();
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db_pool)
        .finish()
}

async fn save_sdl_to_file(schema: &BattlemonSchema) -> Result<()> {
    let path = std::env::current_dir()
        .context("Failed to determine the current directory")?
        .parent()
        .context("Failed to get parent directory")?
        .join("indexer/schema.graphql");

    let mut file = File::create(path).await.context("Failed to create file.")?;
    file.write_all(schema.sdl().as_bytes())
        .await
        .context("Failed to write data with graphql schema in file.")?;

    Ok(())
}
