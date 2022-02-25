pub enum EnvKey {
    AppName,
    DatabaseUrl,
}

pub fn from_env(env_key: EnvKey) -> &'static str {
    match env_key {
        EnvKey::AppName => dotenv!("APP_NAME"),
        EnvKey::DatabaseUrl => dotenv!("DATABASE_URL"),
    }
}
