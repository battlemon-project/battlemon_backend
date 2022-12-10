use async_graphql::SimpleObject;
use serde::Deserialize;
use sqlx::types::Json;

#[derive(SimpleObject, Debug)]
pub struct NftToken {
    pub id: String,
    pub r#type: String,
    pub owner: String,
    pub url: String,
    // pub traits: Json<Vec<Trait>>,
}

#[derive(Deserialize, Debug)]
pub struct Trait {
    name: String,
    flavor: String,
}
