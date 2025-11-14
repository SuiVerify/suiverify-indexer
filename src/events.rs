use serde::Deserialize;
use sui_types::base_types::{ObjectID, SuiAddress};

#[derive(Deserialize, Debug, Clone)]
pub struct DIDClaimed {
    pub registry_id: ObjectID,
    pub user_address: SuiAddress,
    pub did_type: u8,
    pub user_did_id: ObjectID,
    pub nft_id: ObjectID,
}
