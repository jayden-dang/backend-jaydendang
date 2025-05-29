// Models module 

use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress},
    coin::Coin,
    dynamic_field::DynamicFieldInfo,
    object::ObjectRead,
    sui_object::SuiObjectData,
};

#[derive(Debug)]
pub struct CoinBalance {
    pub coin_type: String,
    pub coin_object_count: usize,
    pub total_balance: u64,
    pub locked_balance: std::collections::HashMap<ObjectID, u64>,
}

#[derive(Debug)]
pub struct ObjectInfo {
    pub object_id: ObjectID,
    pub version: u64,
    pub digest: String,
    pub type_: String,
    pub owner: String,
    pub previous_transaction: String,
    pub storage_rebate: u64,
    pub content: Option<SuiObjectData>,
}

#[derive(Debug)]
pub struct DynamicFieldPage {
    pub data: Vec<DynamicFieldInfo>,
    pub next_cursor: Option<ObjectID>,
    pub has_next_page: bool,
} 