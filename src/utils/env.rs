pub enum EnvKey {
    JwtSecret,
    AppName,
    DatabaseUrl,
}

pub fn from_env(env_key: EnvKey) -> &'static str {
    match env_key {
        EnvKey::AppName => dotenv!("APP_NAME"),
        EnvKey::JwtSecret => dotenv!("JWT_SECRET"),
        EnvKey::DatabaseUrl => dotenv!("DATABASE_URL"),
    }
}
