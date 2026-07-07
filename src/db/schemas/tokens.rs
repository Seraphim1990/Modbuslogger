
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub user_id: i32,
    pub user_role_id: i32,
    pub token_hash: String,
    pub created_at: i64,
    pub expires_at: i64
}
