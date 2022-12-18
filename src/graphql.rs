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

    async fn nft_token(&self, ctx: &Context<'_>, id: String) -> Result<NftToken> {
        let pool = ctx.data_unchecked::<PgPool>();
        let token = get_nft_token_db(id, pool)
            .await
            .context("Failed to get nft token data from database")?;

        Ok(token)
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

#[tracing::instrument(name = "Query nft tokens from database", skip(pool))]
async fn get_nft_token_db(id: String, pool: &PgPool) -> StdResult<NftToken, sqlx::Error> {
    query_as!(
        NftToken,
        r#"
        SELECT id, type, owner, url FROM nft_tokens WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}
