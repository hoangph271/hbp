use std::path::PathBuf;

use super::constants;

pub enum EnvKey {
    AppName,
    JwtSecret,
    JwtExpiresInHours,
    MarpApiRoot,
    AstraUri,
    AstraBearerToken,
    AstraKeySpace,
    PublicFilesRoot,
    RootUser,
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
        EnvKey::AstraKeySpace => dotenv!("ASTRA_KEY_SPACE"),
        EnvKey::PublicFilesRoot => dotenv!("PUBLIC_FILES_ROOT"),
        EnvKey::RootUser => dotenv!("ROOT_USER"),
    }
}

pub fn public_files_root() -> PathBuf {
    let path: PathBuf = from_env(EnvKey::PublicFilesRoot).into();

    path
}
pub fn is_root(sub: &str) -> bool {
    if sub.is_empty() {
        return false;
    }

    let root_user = from_env(EnvKey::RootUser);

    root_user.eq(sub)
}
