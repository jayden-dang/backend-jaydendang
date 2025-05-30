use crate::domain::{AuthUser, AuthUserFilter, AuthUserForCreate, AuthUserForUpdate};
use crate::error::{Error, Result};
use crate::AuthUserDmc;
use async_trait::async_trait;
use jd_core::{base::rest, ModelManager};
use modql::filter::OpValsString;

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create_user(&self, user: &AuthUser) -> Result<()>;
  async fn get_user(&self, address: &str) -> Result<Option<AuthUser>>;
  async fn update_user(&self, user: &AuthUser) -> Result<()>;
  async fn update_login(&self, address: &str) -> Result<()>;
}

pub struct RestUserRepository {
  model_manager: ModelManager,
}

impl RestUserRepository {
  pub fn new(model_manager: ModelManager) -> Self {
    Self { model_manager }
  }
}

#[async_trait]
impl UserRepository for RestUserRepository {
  async fn create_user(&self, user: &AuthUser) -> Result<()> {
    let input = user.clone().into_create_input();

    let _created: AuthUser = rest::create::<AuthUserDmc, _, _>(&self.model_manager, input)
      .await
      .map_err(|e| Error::database_error(&e.as_ref()))?;

    Ok(())
  }

  async fn get_user(&self, address: &str) -> Result<Option<AuthUser>> {
    let filter = AuthUserFilter { address: Some(address.to_string().into()) };

    match rest::get_by_sth::<AuthUserDmc, _, AuthUser>(&self.model_manager, Some(filter)).await {
      Ok(user) => Ok(Some(user)),
      Err(e) => match e {
        jd_core::Error::EntityNotFound { .. } => Ok(None),
        _ => Err(Error::database_error(&e.as_ref())),
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
      rest::update_by_filter::<AuthUserDmc, _, _>(&self.model_manager, filter, update_input)
        .await
        .map_err(|e| Error::database_error(&e.to_string()))?;

    if updated_count == 0 {
      return Err(Error::database_error("User not found for update"));
    }

    Ok(())
  }

  async fn update_login(&self, address: &str) -> Result<()> {
    // First get the current user to increment login count
    if let Some(mut user) = self.get_user(address).await? {
      user.update_login();
      self.update_user(&user).await?;
    } else {
      return Err(Error::database_error("User not found for login update"));
    }

    Ok(())
  }
}
