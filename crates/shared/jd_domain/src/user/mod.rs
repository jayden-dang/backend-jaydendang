use uuid::Uuid;

pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
}
