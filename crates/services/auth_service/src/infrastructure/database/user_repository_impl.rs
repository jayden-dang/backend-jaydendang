use async_trait::async_trait;
use jd_core::{AppState, base::rest};

use crate::AuthUserDmc;
use crate::domain::{AuthUser, AuthUserFilter, AuthUserForUpdate, UserRepository};
use crate::error::{Error, Result};

pub struct UserRepositoryImpl {
  state: AppState,
}

impl UserRepositoryImpl {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
  async fn create_user(&self, user: &AuthUser) -> Result<()> {
    let input = user.clone().into_create_input();

    let _created: AuthUser = rest::create::<AuthUserDmc, _, _>(&self.state.mm, input)
      .await
      .map_err(|e| Error::database_error(e.as_ref()))?;

    Ok(())
  }

  async fn get_user(&self, address: &str) -> Result<Option<AuthUser>> {
    let filter = AuthUserFilter { address: Some(address.to_string().into()) };

    match rest::get_by_sth::<AuthUserDmc, _, AuthUser>(&self.state.mm, Some(filter)).await {
      Ok(user) => Ok(Some(user)),
      Err(e) => match e {
        jd_core::Error::EntityNotFound { .. } => Ok(None),
        _ => Err(Error::database_error(e.as_ref())),
      },
    }
  }

  async fn update_user(&self, user: &AuthUser) -> Result<()> {
    let update_input = AuthUserForUpdate {
      public_key: Some(user.public_key.clone()),
      last_login: Some(user.last_login),
      login_count: Some(user.login_count),
    };

    let filter = AuthUserFilter { address: Some(user.address.clone().into()) };

    let updated_count =
      rest::update_by_filter::<AuthUserDmc, _, _>(&self.state.mm, filter, update_input)
        .await
        .map_err(|e| Error::database_error(e.as_ref()))?;

    if updated_count == 0 {
      return Err(Error::database_error("User not found for update"));
    }

    Ok(())
  }
}
