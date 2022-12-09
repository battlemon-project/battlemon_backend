use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn add(&self, a: u32, b: u32) -> u32 {
        a + b
    }
}
