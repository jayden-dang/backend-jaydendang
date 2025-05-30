use jd_contracts::user::dtos::requests::{
  create_user_request::CreateUserRequest, user_filter::UserFilter,
};
use jd_core::ctx::Ctx;
use jd_core::{AppState, Result};
use serde_json::{Value, json};
use user_service::{
  application::use_cases::{CreateUserUseCase, GetUserUseCase},
  infrastructure::database::user_repository_impl::UserRepositoryImpl,
};

// Note: For proper RPC integration, we should pass AppState instead of ModelManager
// For now, we'll work with what we have, but this is a design limitation

// Real RPC handler using user_service
pub async fn handle_user_rpc(
  method: &str,
  params: Value,
  _ctx: Ctx,
  app_state: AppState,
) -> Result<Value> {
  match method {
    "get_user_by_username" => {
      let username = params
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| jd_core::Error::RpcError("Missing username parameter".to_string()))?;

      let repository = UserRepositoryImpl::new(app_state.clone());
      let use_case = GetUserUseCase::new(repository);

      let user = use_case
        .execute_by_username(username.to_string())
        .await
        .map_err(|e| jd_core::Error::RpcError(format!("Failed to get user: {}", e)))?;

      Ok(json!({ "data": user }))
    }
    "get_user_by_email" => {
      let email = params
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| jd_core::Error::RpcError("Missing email parameter".to_string()))?;

      let repository = UserRepositoryImpl::new(app_state.clone());
      let use_case = GetUserUseCase::new(repository);

      let user = use_case
        .execute_by_email(email.to_string())
        .await
        .map_err(|e| jd_core::Error::RpcError(format!("Failed to get user: {}", e)))?;

      Ok(json!({ "data": user }))
    }
    "get_user_by_filter" => {
      let filter_data = params
        .get("filter")
        .ok_or_else(|| jd_core::Error::RpcError("Missing filter parameter".to_string()))?;

      let filter: UserFilter = serde_json::from_value(filter_data.clone())
        .map_err(|e| jd_core::Error::RpcError(format!("Invalid filter format: {}", e)))?;

      let repository = UserRepositoryImpl::new(app_state.clone());
      let use_case = GetUserUseCase::new(repository);

      let user = use_case
        .execute_by_wow(filter)
        .await
        .map_err(|e| jd_core::Error::RpcError(format!("Failed to get user: {}", e)))?;

      Ok(json!({ "data": user }))
    }
    "create_user" => {
      let data: CreateUserRequest = serde_json::from_value(
        params
          .get("data")
          .ok_or_else(|| jd_core::Error::RpcError("Missing data parameter".to_string()))?
          .clone(),
      )
      .map_err(|e| jd_core::Error::RpcError(format!("Invalid data format: {}", e)))?;

      let repository = UserRepositoryImpl::new(app_state.clone());
      let use_case = CreateUserUseCase::new(repository);

      let user = use_case
        .execute(data)
        .await
        .map_err(|e| jd_core::Error::RpcError(format!("Failed to create user: {}", e)))?;

      Ok(json!({ "data": user }))
    }
    "get_user_by_active_status" => {
      let is_active = params
        .get("is_active")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| jd_core::Error::RpcError("Missing is_active parameter".to_string()))?;

      let repository = UserRepositoryImpl::new(app_state.clone());
      let use_case = GetUserUseCase::new(repository);

      let user = use_case
        .execute_by_is_active(is_active)
        .await
        .map_err(|e| jd_core::Error::RpcError(format!("Failed to get user: {}", e)))?;

      Ok(json!({ "data": user }))
    }
    _ => Err(jd_core::Error::RpcError(format!("Unknown method: {}", method))),
  }
}
