use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // For future use

use jd_core::ctx::Ctx;
use auth_service::domain::{UserRole, UserPermission};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub permissions: Vec<UserPermission>,
    pub is_active: bool,
    pub session_id: Option<uuid::Uuid>,
}

impl AuthContext {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions
            .iter()
            .any(|p| p.permission_name == permission)
    }

    pub fn can_access_resource(&self, resource: &str, action: &str) -> bool {
        self.permissions
            .iter()
            .any(|p| p.resource == resource && p.action == action)
    }

    pub fn has_role(&self, required_role: UserRole) -> bool {
        self.role.can_access(required_role)
    }

    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    pub fn is_staff(&self) -> bool {
        self.role.is_staff()
    }
}

pub async fn mw_require_auth(
    State(ctx): State<Ctx>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_context = extract_auth_context(&ctx, &req).await?;
    
    if !auth_context.is_active {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add auth context to request extensions
    req.extensions_mut().insert(auth_context);
    
    Ok(next.run(req).await)
}

pub fn require_role(required_role: UserRole) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req: Request, next: Next| {
        let role = required_role;
        Box::pin(async move {
            let auth_context = req
                .extensions()
                .get::<AuthContext>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if !auth_context.has_role(role) {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        })
    }
}

pub fn require_permission(permission: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req: Request, next: Next| {
        Box::pin(async move {
            let auth_context = req
                .extensions()
                .get::<AuthContext>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if !auth_context.has_permission(permission) {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        })
    }
}

pub fn require_resource_access(resource: &'static str, action: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req: Request, next: Next| {
        Box::pin(async move {
            let auth_context = req
                .extensions()
                .get::<AuthContext>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if !auth_context.can_access_resource(resource, action) {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        })
    }
}

async fn extract_auth_context(ctx: &Ctx, req: &Request) -> Result<AuthContext, StatusCode> {
    // Extract JWT token from Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate JWT and extract user info
    // This would use your JWT validation logic from auth_service
    validate_jwt_and_get_context(ctx, token).await
}

async fn validate_jwt_and_get_context(_ctx: &Ctx, _token: &str) -> Result<AuthContext, StatusCode> {
    // This is a placeholder - you would implement the actual JWT validation
    // and database lookup here using your auth_service
    
    // Example implementation outline:
    // 1. Validate JWT signature and claims
    // 2. Extract user_id from JWT
    // 3. Query database for user and permissions
    // 4. Build AuthContext
    
    // For now, return a mock context for testing
    // TODO: Implement actual JWT validation
    Err(StatusCode::UNAUTHORIZED)
}

// Convenience macros for common role checks
#[macro_export]
macro_rules! require_admin {
    () => {
        $crate::middleware::mw_auth_rbac::require_role(auth_service::domain::UserRole::Admin)
    };
}

#[macro_export]
macro_rules! require_moderator {
    () => {
        $crate::middleware::mw_auth_rbac::require_role(auth_service::domain::UserRole::Moderator)
    };
}

#[macro_export]
macro_rules! require_member {
    () => {
        $crate::middleware::mw_auth_rbac::require_role(auth_service::domain::UserRole::Member)
    };
}

// Helper trait for extracting auth context from requests
pub trait AuthContextExt {
    fn auth_context(&self) -> Option<&AuthContext>;
    fn require_auth_context(&self) -> Result<&AuthContext, StatusCode>;
}

impl AuthContextExt for Request {
    fn auth_context(&self) -> Option<&AuthContext> {
        self.extensions().get::<AuthContext>()
    }

    fn require_auth_context(&self) -> Result<&AuthContext, StatusCode> {
        self.auth_context().ok_or(StatusCode::UNAUTHORIZED)
    }
}

// Usage examples:
/*
use axum::{Router, routing::get, middleware};

fn protected_routes() -> Router {
    Router::new()
        .route("/admin", get(admin_handler))
        .layer(middleware::from_fn(require_admin!()))
        .route("/moderate", get(moderate_handler))
        .layer(middleware::from_fn(require_moderator!()))
        .route("/member", get(member_handler))
        .layer(middleware::from_fn(require_member!()))
        .route("/user-read", get(user_read_handler))
        .layer(middleware::from_fn(require_permission("users.read.own")))
        .route("/content-write", get(content_write_handler))
        .layer(middleware::from_fn(require_resource_access("content", "write")))
        .layer(middleware::from_fn_with_state(ctx.clone(), mw_require_auth))
}

async fn admin_handler(req: Request) -> impl IntoResponse {
    let auth = req.require_auth_context().unwrap();
    format!("Hello Admin {}!", auth.username)
}
*/