use crate::{
  Error, Result,
  application::use_cases::{CreateUserUseCase, GetUserUseCase},
  domain::user_repository_trait::UserRepository,
  infrastructure::database::user_repository_impl::UserRepositoryImpl,
};
use axum::{
  Json,
  extract::{Path, Query, State},
};
use jd_contracts::user::dtos::{
  records::user_record::UserRecord,
  requests::{
    create_profile_request::CreateUserProfileRequest, create_user_request::CreateUserRequest,
    user_filter::UserFilter,
  },
};
use jd_core::AppState;
use tracing::error;

use std::sync::Arc;

pub struct UserHandler<R: UserRepository> {
  pub create_user: CreateUserUseCase<R>,
  pub get_user: GetUserUseCase<R>,
}

impl<R: UserRepository> UserHandler<R> {
  pub fn new(create_user: CreateUserUseCase<R>, get_user: GetUserUseCase<R>) -> Self {
    Self { create_user, get_user }
  }

  pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
  ) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state.clone());
    let use_case = CreateUserUseCase::new(repository);

    state
      .mm
      .dbx()
      .begin_txn()
      .await
      .map_err(|e| Error::from(Arc::new(e)))?;

    let result = async {
      let user = use_case.execute(request).await?;
      let profile_request = CreateUserProfileRequest::with_defaults(user.user_id.clone());
      let _profile = use_case.execute_create_profile(profile_request).await?;
      Ok(user)
    }
    .await;

    match result {
      Ok(user) => {
        state.mm.dbx().commit_txn().await.map_err(|e| {
          error!("Failed to commit transaction: {:?}", e);
          Error::AccessDenied { resource: e.to_string() }
        })?;
        Ok(Json(user))
      }
      Err(e) => {
        if let Err(rollback_err) = state.mm.dbx().rollback_txn().await {
          error!("Failed to commit transaction: {:?}", rollback_err);
        }

        Err(e)
      }
    }
  }

  pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(id): Path<String>,
  ) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = GetUserUseCase::new(repository);
    let result = use_case.execute_by_username(id).await?;
    Ok(Json(result))
  }

  pub async fn get_user_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
  ) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = GetUserUseCase::new(repository);
    let result = use_case.execute_by_email(email).await?;
    Ok(Json(result))
  }

  pub async fn get_user_by_wow(
    State(state): State<AppState>,
    Query(query): Query<UserFilter>,
  ) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = GetUserUseCase::new(repository);
    let result = use_case.execute_by_wow(query).await?;
    Ok(Json(result))
  }
}
