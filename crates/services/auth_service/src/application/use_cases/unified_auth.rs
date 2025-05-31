use uuid::Uuid;
use time::OffsetDateTime;

use crate::domain::{
    UnifiedAuthUser, UnifiedAuthUserForCreate, UnifiedAuthUserForUpdate,
    UserAuthProvider, UserAuthProviderForCreate,
    AuthProviderType, ProviderStatus, UserRole, OAuthTokenResponse
};
use crate::infrastructure::oauth::{OAuthClient, OAuthUserInfo};
use crate::error::{Result, Error};

pub struct UnifiedAuthService {
    oauth_client: OAuthClient,
    // Would also have database repositories here
}

#[derive(Debug, Clone)]
pub struct LoginResult {
    pub user: UnifiedAuthUser,
    pub jwt_token: String,
    pub refresh_token: Option<String>,
    pub is_new_user: bool,
}

#[derive(Debug, Clone)]
pub struct AuthProviderResult {
    pub provider: UserAuthProvider,
    pub is_new_provider: bool,
}

impl UnifiedAuthService {
    pub fn new(oauth_client: OAuthClient) -> Self {
        Self {
            oauth_client,
        }
    }

    // OAuth Authentication Flow
    pub async fn initiate_oauth_login(
        &self,
        provider_type: AuthProviderType,
        redirect_url: Option<String>,
    ) -> Result<String> {
        let provider = self.oauth_client
            .get_provider(provider_type)
            .ok_or(Error::unsupported_oauth_provider())?;

        let state = OAuthClient::generate_state(provider_type, redirect_url);
        let auth_url = provider.get_authorization_url(&state);

        Ok(auth_url)
    }

    pub async fn complete_oauth_login(
        &self,
        provider_type: AuthProviderType,
        code: String,
        state: String,
    ) -> Result<LoginResult> {
        // Handle OAuth callback
        let (token_response, user_info, _oauth_state) = self
            .oauth_client
            .handle_oauth_callback(provider_type, code, state)
            .await?;

        // Check if user already exists with this provider
        if let Ok(existing_provider) = self.find_auth_provider_by_external_id(
            provider_type,
            &user_info.id,
        ).await {
            // Update existing provider with new token
            let updated_provider = self.update_oauth_provider_tokens(
                existing_provider.provider_id,
                &token_response,
            ).await?;

            // Get the user and update login info
            let mut user = self.get_user_by_id(updated_provider.user_id).await?;
            user = self.update_user_login(user.user_id).await?;

            let jwt_token = self.generate_jwt_for_user(&user).await?;

            return Ok(LoginResult {
                user,
                jwt_token,
                refresh_token: token_response.refresh_token,
                is_new_user: false,
            });
        }

        // Check if user exists with the same email from another provider
        if let Ok(existing_user) = self.find_user_by_email(&user_info.email).await {
            // Link this OAuth provider to existing user
            let _new_provider = self.create_oauth_provider_for_user(
                existing_user.user_id,
                provider_type,
                &user_info,
                &token_response,
            ).await?;

            // Update user login info
            let user = self.update_user_login(existing_user.user_id).await?;
            let jwt_token = self.generate_jwt_for_user(&user).await?;

            return Ok(LoginResult {
                user,
                jwt_token,
                refresh_token: token_response.refresh_token,
                is_new_user: false,
            });
        }

        // Create new user with OAuth provider
        let new_user = self.create_user_with_oauth_provider(
            provider_type,
            &user_info,
            &token_response,
        ).await?;

        let jwt_token = self.generate_jwt_for_user(&new_user).await?;

        Ok(LoginResult {
            user: new_user,
            jwt_token,
            refresh_token: token_response.refresh_token,
            is_new_user: true,
        })
    }

    // Email/Password Authentication
    pub async fn register_with_email(
        &self,
        email: String,
        username: String,
        password: String,
        display_name: Option<String>,
    ) -> Result<LoginResult> {
        // Check if email already exists
        if self.find_user_by_email(&email).await.is_ok() {
            return Err(Error::email_already_exists());
        }

        // Check if username already exists
        if self.find_user_by_username(&username).await.is_ok() {
            return Err(Error::username_already_exists());
        }

        // Hash password
        let password_hash = self.hash_password(&password)?;

        // Create user
        let user_create = UnifiedAuthUserForCreate {
            email: Some(email.clone()),
            username: username.clone(),
            display_name,
            role: Some(UserRole::Normal),
            is_active: Some(true),
            is_email_verified: Some(false),
        };

        let user = self.create_user(user_create).await?;

        // Create email auth provider
        let provider_create = UserAuthProvider::new_email_provider(
            user.user_id,
            email,
            password_hash,
        );

        self.create_auth_provider(provider_create).await?;

        let jwt_token = self.generate_jwt_for_user(&user).await?;

        Ok(LoginResult {
            user,
            jwt_token,
            refresh_token: None,
            is_new_user: true,
        })
    }

    pub async fn login_with_email(
        &self,
        email: String,
        password: String,
    ) -> Result<LoginResult> {
        // Find user by email
        let user = self.find_user_by_email(&email).await?;

        if !user.is_active {
            return Err(Error::account_disabled());
        }

        // Find email auth provider
        let provider = self.find_email_provider_for_user(user.user_id).await?;

        if !matches!(provider.status, ProviderStatus::Active) {
            return Err(Error::account_disabled());
        }

        // Verify password
        let password_hash = provider.password_hash
            .ok_or(Error::invalid_credentials())?;

        if !self.verify_password(&password, &password_hash)? {
            return Err(Error::invalid_credentials());
        }

        // Update login info
        let user = self.update_user_login(user.user_id).await?;
        let jwt_token = self.generate_jwt_for_user(&user).await?;

        Ok(LoginResult {
            user,
            jwt_token,
            refresh_token: None,
            is_new_user: false,
        })
    }

    // Wallet Authentication
    pub async fn login_with_wallet(
        &self,
        wallet_address: String,
        public_key: String,
        signature: String,
        nonce: String,
    ) -> Result<LoginResult> {
        // Verify wallet signature
        self.verify_wallet_signature(&wallet_address, &public_key, &signature, &nonce).await?;

        // Check if user exists with this wallet
        if let Ok(existing_provider) = self.find_wallet_provider_by_address(&wallet_address).await {
            let mut user = self.get_user_by_id(existing_provider.user_id).await?;
            user = self.update_user_login(user.user_id).await?;

            let jwt_token = self.generate_jwt_for_user(&user).await?;

            return Ok(LoginResult {
                user,
                jwt_token,
                refresh_token: None,
                is_new_user: false,
            });
        }

        // Create new user with wallet
        let username = self.generate_username_from_wallet(&wallet_address);
        let user_create = UnifiedAuthUserForCreate {
            email: None,
            username,
            display_name: None,
            role: Some(UserRole::Normal),
            is_active: Some(true),
            is_email_verified: Some(false),
        };

        let user = self.create_user(user_create).await?;

        // Create wallet auth provider
        let provider_create = UserAuthProvider::new_wallet_provider(
            user.user_id,
            wallet_address,
            public_key,
        );

        self.create_auth_provider(provider_create).await?;

        let jwt_token = self.generate_jwt_for_user(&user).await?;

        Ok(LoginResult {
            user,
            jwt_token,
            refresh_token: None,
            is_new_user: true,
        })
    }

    // Provider Management
    pub async fn add_auth_provider_to_user(
        &self,
        _user_id: Uuid,
        _provider_type: AuthProviderType,
        // Provider-specific data would be passed here
    ) -> Result<AuthProviderResult> {
        // Check if user already has this provider type
        if self.user_has_provider_type(_user_id, _provider_type).await? {
            return Err(Error::invalid_request_data("Provider already linked"));
        }

        // Implementation would depend on provider type
        // For OAuth, would need to initiate OAuth flow
        // For email, would need email and password
        // For wallet, would need wallet verification

        todo!("Implement provider-specific linking logic")
    }

    pub async fn remove_auth_provider(
        &self,
        user_id: Uuid,
        provider_id: Uuid,
    ) -> Result<()> {
        // Check that user has other active providers before removing
        let provider_count = self.count_active_providers_for_user(user_id).await?;
        
        if provider_count <= 1 {
            return Err(Error::invalid_request_data("Cannot remove last authentication provider"));
        }

        self.deactivate_auth_provider(provider_id).await?;
        Ok(())
    }

    pub async fn get_user_providers(&self, user_id: Uuid) -> Result<Vec<UserAuthProvider>> {
        self.list_providers_for_user(user_id).await
    }

    // Helper methods (these would be implemented using your database repositories)
    async fn find_auth_provider_by_external_id(
        &self,
        _provider_type: AuthProviderType,
        _external_id: &str,
    ) -> Result<UserAuthProvider> {
        // Implementation would query the database
        todo!("Implement database query")
    }

    async fn find_user_by_email(&self, _email: &str) -> Result<UnifiedAuthUser> {
        todo!("Implement database query")
    }

    async fn find_user_by_username(&self, _username: &str) -> Result<UnifiedAuthUser> {
        todo!("Implement database query")
    }

    async fn get_user_by_id(&self, _user_id: Uuid) -> Result<UnifiedAuthUser> {
        todo!("Implement database query")
    }

    async fn create_user(&self, _user: UnifiedAuthUserForCreate) -> Result<UnifiedAuthUser> {
        todo!("Implement database insert")
    }

    async fn update_user_login(&self, _user_id: Uuid) -> Result<UnifiedAuthUser> {
        let _update = UnifiedAuthUserForUpdate {
            email: None,
            username: None,
            display_name: None,
            role: None,
            is_active: None,
            is_email_verified: None,
            is_profile_complete: None,
            last_login: Some(OffsetDateTime::now_utc()),
            login_count: None, // Would increment in database
        };

        todo!("Implement database update")
    }

    async fn create_auth_provider(&self, _provider: UserAuthProviderForCreate) -> Result<UserAuthProvider> {
        todo!("Implement database insert")
    }

    async fn create_oauth_provider_for_user(
        &self,
        user_id: Uuid,
        provider_type: AuthProviderType,
        user_info: &OAuthUserInfo,
        token_response: &OAuthTokenResponse,
    ) -> Result<UserAuthProvider> {
        let provider_create = UserAuthProvider::new_oauth_provider(
            user_id,
            provider_type,
            user_info.id.clone(),
            user_info.email.clone(),
            token_response.clone(),
            Some(user_info.raw_data.clone()),
        );

        self.create_auth_provider(provider_create).await
    }

    async fn create_user_with_oauth_provider(
        &self,
        provider_type: AuthProviderType,
        user_info: &OAuthUserInfo,
        token_response: &OAuthTokenResponse,
    ) -> Result<UnifiedAuthUser> {
        // Generate unique username from email or name
        let username = self.generate_unique_username(&user_info.email, user_info.name.as_deref()).await?;

        let user_create = UnifiedAuthUserForCreate {
            email: Some(user_info.email.clone()),
            username,
            display_name: user_info.name.clone(),
            role: Some(UserRole::Normal),
            is_active: Some(true),
            is_email_verified: Some(user_info.verified_email.unwrap_or(true)),
        };

        let user = self.create_user(user_create).await?;

        // Create OAuth provider
        self.create_oauth_provider_for_user(
            user.user_id,
            provider_type,
            user_info,
            token_response,
        ).await?;

        Ok(user)
    }

    async fn generate_unique_username(&self, email: &str, name: Option<&str>) -> Result<String> {
        let base_username = if let Some(name) = name {
            name.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        } else {
            email.split('@').next().unwrap_or("user").to_string()
        };

        let mut username = base_username.clone();
        let mut counter = 1;

        while self.find_user_by_username(&username).await.is_ok() {
            username = format!("{}{}", base_username, counter);
            counter += 1;
        }

        Ok(username)
    }

    fn generate_username_from_wallet(&self, wallet_address: &str) -> String {
        format!("wallet_{}", &wallet_address[2..10]) // Use first 8 chars after 0x
    }

    async fn generate_jwt_for_user(&self, _user: &UnifiedAuthUser) -> Result<String> {
        // Implementation would use your JWT generation logic
        todo!("Implement JWT generation")
    }

    fn hash_password(&self, _password: &str) -> Result<String> {
        // Implementation would use bcrypt or argon2
        todo!("Implement password hashing")
    }

    fn verify_password(&self, _password: &str, _hash: &str) -> Result<bool> {
        // Implementation would use bcrypt or argon2
        todo!("Implement password verification")
    }

    async fn verify_wallet_signature(
        &self,
        _wallet_address: &str,
        _public_key: &str,
        _signature: &str,
        _nonce: &str,
    ) -> Result<()> {
        // Implementation would verify the cryptographic signature
        todo!("Implement signature verification")
    }

    // Additional helper methods...
    async fn update_oauth_provider_tokens(
        &self,
        _provider_id: Uuid,
        _token_response: &OAuthTokenResponse,
    ) -> Result<UserAuthProvider> {
        todo!("Implement token update")
    }

    async fn find_email_provider_for_user(&self, _user_id: Uuid) -> Result<UserAuthProvider> {
        todo!("Implement database query")
    }

    async fn find_wallet_provider_by_address(&self, _address: &str) -> Result<UserAuthProvider> {
        todo!("Implement database query")
    }

    async fn user_has_provider_type(&self, _user_id: Uuid, _provider_type: AuthProviderType) -> Result<bool> {
        todo!("Implement database query")
    }

    async fn count_active_providers_for_user(&self, _user_id: Uuid) -> Result<usize> {
        todo!("Implement database query")
    }

    async fn deactivate_auth_provider(&self, _provider_id: Uuid) -> Result<()> {
        todo!("Implement database update")
    }

    async fn list_providers_for_user(&self, _user_id: Uuid) -> Result<Vec<UserAuthProvider>> {
        todo!("Implement database query")
    }
}