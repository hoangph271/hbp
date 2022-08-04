use super::constants;

pub enum EnvKey {
    AppName,
    JwtSecret,
    JwtExpiresInHours,
    MarpApiRoot,
    AstraUri,
    AstraBearerToken,
}

pub fn from_env(env_key: EnvKey) -> &'static str {
    match env_key {
        EnvKey::AppName => dotenv!("APP_NAME"),
        EnvKey::JwtSecret => dotenv!("JWT_SECRET"),
        EnvKey::JwtExpiresInHours => {
            let from_env: &str = dotenv!("JWT_EXPRIES_IN_HOURS");

            if from_env.is_empty() {
                constants::DEFAULT_JWT_EXPIRES_IN
            } else {
                from_env
            }
        }
        EnvKey::MarpApiRoot => dotenv!("MARP_API_ROOT"),
        EnvKey::AstraUri => dotenv!("ASTRA_URI"),
        EnvKey::AstraBearerToken => dotenv!("ASTRA_BEARER_TOKEN"),
    }
}
