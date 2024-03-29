use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct CreateUserWithAdmin {
    pub admin_id: String,
    pub admin_password: String,
    pub user_info: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteUsersWithAdmin {
    pub admin_id: String,
    pub admin_password: String,
    pub users_ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub user_password: String,
    pub role: Vec<Role>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Role {
    // Admin,
    User,
    Read,
    Write,
    History,
}

#[derive(Serialize, Deserialize)]
pub struct UserId {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub user_password: String,
}
