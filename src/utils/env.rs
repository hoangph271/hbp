pub enum EnvKey {
    JwtSecret,
    AppName,
    MarpApiRoot,
    AstraUri,
    AstraBearerToken
}

pub fn from_env(env_key: EnvKey) -> &'static str {
    match env_key {
        EnvKey::AppName => dotenv!("APP_NAME"),
        EnvKey::JwtSecret => dotenv!("JWT_SECRET"),
        EnvKey::MarpApiRoot => dotenv!("MARP_API_ROOT"),
        EnvKey::AstraUri => dotenv!("ASTRA_URI"),
        EnvKey::AstraBearerToken => dotenv!("ASTRA_BEARER_TOKEN"),
    }
}
