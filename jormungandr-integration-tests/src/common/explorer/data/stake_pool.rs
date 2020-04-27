use super::{GraphQLQuery, GraphQLResponse};
use jormungandr_lib::crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplorerStakePool {
    pub id: Hash,
    pub registration: ExplorerPoolRegistration,
    pub retirement: Option<ExplorerPoolRetirement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplorerPoolRetirement {
    pub id: Hash,
}

impl TryFrom<serde_json::Value> for ExplorerPoolRetirement {
    type Error = serde_json::Error;

    fn try_from(
        part_of_response: serde_json::Value,
    ) -> Result<ExplorerPoolRetirement, Self::Error> {
        Ok(ExplorerPoolRetirement {
            id: serde_json::from_str(&part_of_response["poolId"].to_string())?,
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplorerPoolRegistration {
    pub id: Hash,
}

impl TryFrom<serde_json::Value> for ExplorerPoolRegistration {
    type Error = serde_json::Error;

    fn try_from(
        part_of_response: serde_json::Value,
    ) -> Result<ExplorerPoolRegistration, Self::Error> {
        Ok(ExplorerPoolRegistration {
            id: serde_json::from_str(&part_of_response["pool"]["id"].to_string())?,
        })
    }
}

impl TryFrom<GraphQLResponse> for ExplorerStakePool {
    type Error = serde_json::Error;

    fn try_from(response: GraphQLResponse) -> Result<ExplorerStakePool, Self::Error> {
        let retirement_response = response.data["stakePool"]["retirement"].clone();
        let retirement = {
            if retirement_response.is_null() {
                None
            } else {
                Some(ExplorerPoolRetirement::try_from(retirement_response)?)
            }
        };

        Ok(ExplorerStakePool {
            id: serde_json::from_str(&response.data["stakePool"]["id"].to_string())?,
            registration: ExplorerPoolRegistration::try_from(
                response.data["stakePool"]["registration"].clone(),
            )?,
            retirement: retirement,
        })
    }
}

impl ExplorerStakePool {
    pub fn query_by_id(hash: Hash) -> GraphQLQuery {
        GraphQLQuery {
            query: format!(
                r#"{{ stakePool(id: "{}") {{
            id
            registration {{
                pool {{
                    id
                }}
            }}
            retirement {{
                poolId
            }}
        }}
}}"#,
                hash.into_hash()
            ),
        }
    }
}
