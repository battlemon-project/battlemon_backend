mod nft_token;

use crate::graphql::nft_token::NftToken;
use async_graphql::{Context, Object};
use color_eyre::eyre::{Result, WrapErr};
use sqlx::{query_as, PgPool};
use std::result::Result as StdResult;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn nft_tokens(&self, ctx: &Context<'_>) -> Result<Vec<NftToken>> {
        let pool = ctx.data_unchecked::<PgPool>();
        let tokens = get_nft_tokens_db(pool)
            .await
            .context("Failed to get nft tokens data from database")?;

        Ok(tokens)
    }
}

#[tracing::instrument(name = "Query nft tokens from database", skip_all)]
async fn get_nft_tokens_db(pool: &PgPool) -> StdResult<Vec<NftToken>, sqlx::Error> {
    query_as!(
        NftToken,
        r#"
        SELECT id, type, owner, url FROM nft_tokens
        "#
    )
    .fetch_all(pool)
    .await
}
