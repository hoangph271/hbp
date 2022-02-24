
pub enum EnvKey {
    AppName,
}

pub fn from_env(env_key: EnvKey) -> &'static str {
    match env_key {
        EnvKey::AppName => {
            dotenv!("APP_NAME")
        }
    }
}
