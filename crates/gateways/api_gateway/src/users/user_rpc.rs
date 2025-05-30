use jd_core::ModelManager;
use jd_core::ctx::Ctx;
use jd_core::Result;
use serde::{Deserialize, Serialize};
use modql::filter::{FilterNodes, OpValsString, OpValsInt64};
use serde_json::{json, Value};

// Simple types for RPC demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRpcEntity {
    pub id: i64,
    pub wallet_address: String,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserForCreate {
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserForUpdate {
    pub wallet_address: Option<String>,
}

#[derive(FilterNodes, Default, Deserialize)]
pub struct UserFilter {
    pub id: Option<OpValsInt64>,
    pub wallet_address: Option<OpValsString>,
}

// Simple manual RPC handler
pub async fn handle_user_rpc(method: &str, params: Value, _ctx: Ctx, _mm: ModelManager) -> Result<Value> {
    match method {
        "get_user" => {
            let id: i64 = params.get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| jd_core::Error::RpcError("Missing id parameter".to_string()))?;
            
            let user = UserRpcEntity {
                id,
                wallet_address: "0x1234567890abcdef".to_string(),
                nonce: Some("test-nonce".to_string()),
            };
            
            Ok(json!({ "data": user }))
        },
        "list_users" => {
            let users = vec![
                UserRpcEntity {
                    id: 1,
                    wallet_address: "0x1234567890abcdef".to_string(),
                    nonce: Some("test-nonce-1".to_string()),
                },
                UserRpcEntity {
                    id: 2,
                    wallet_address: "0xabcdef1234567890".to_string(),
                    nonce: Some("test-nonce-2".to_string()),
                },
            ];
            
            Ok(json!({ "data": users }))
        },
        "create_user" => {
            let data: UserForCreate = serde_json::from_value(
                params.get("data")
                    .ok_or_else(|| jd_core::Error::RpcError("Missing data parameter".to_string()))?
                    .clone()
            ).map_err(|e| jd_core::Error::RpcError(format!("Invalid data format: {}", e)))?;
            
            let user = UserRpcEntity {
                id: 1,
                wallet_address: data.wallet_address,
                nonce: Some("new-nonce".to_string()),
            };
            
            Ok(json!({ "data": user }))
        },
        "update_user" => {
            let _id: i64 = params.get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| jd_core::Error::RpcError("Missing id parameter".to_string()))?;
            
            let data: UserForUpdate = serde_json::from_value(
                params.get("data")
                    .ok_or_else(|| jd_core::Error::RpcError("Missing data parameter".to_string()))?
                    .clone()
            ).map_err(|e| jd_core::Error::RpcError(format!("Invalid data format: {}", e)))?;
            
            let user = UserRpcEntity {
                id: _id,
                wallet_address: data.wallet_address.unwrap_or_else(|| "0xupdated".to_string()),
                nonce: Some("updated-nonce".to_string()),
            };
            
            Ok(json!({ "data": user }))
        },
        "delete_user" => {
            let id: i64 = params.get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| jd_core::Error::RpcError("Missing id parameter".to_string()))?;
            
            let user = UserRpcEntity {
                id,
                wallet_address: "0xdeleted".to_string(),
                nonce: None,
            };
            
            Ok(json!({ "data": user }))
        },
        _ => Err(jd_core::Error::RpcError(format!("Unknown method: {}", method)))
    }
}