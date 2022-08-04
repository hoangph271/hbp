pub mod headers {
    pub const AUTHORIZATION: &str = "authorization";
}
pub mod cookies {
    pub const RESOURCE_JWT: &str = "resource-jwt";
    pub const USER_JWT: &str = "user-jwt";
}
pub const DEFAULT_JWT_EXPIRES_IN: &str = "24";
