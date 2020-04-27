mod last_block;
mod stake_pool;
mod transaction;

pub use last_block::ExplorerLastBlock;
pub use stake_pool::ExplorerStakePool;
pub use transaction::ExplorerTransaction;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphQLResponse {
    data: serde_json::Value,
    errors: Option<serde_json::Value>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphQLQuery {
    query: String,
}
