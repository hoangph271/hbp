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
    FilesRoot,
    RootUser,
    DeployEnv,
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
        EnvKey::FilesRoot => dotenv!("FILES_ROOT"),
        EnvKey::RootUser => dotenv!("ROOT_USER"),
        EnvKey::DeployEnv => dotenv!("DEPLOY_ENV"),
    }
}

pub fn public_files_root() -> PathBuf {
    from_env(EnvKey::PublicFilesRoot).into()
}
pub fn files_root() -> PathBuf {
    from_env(EnvKey::FilesRoot).into()
}
pub fn is_root(username: &str) -> bool {
    if username.is_empty() {
        return false;
    }

    let root_user = from_env(EnvKey::RootUser);

    root_user.eq(username)
}
pub fn jwt_secret() -> Vec<u8> {
    let key = from_env(EnvKey::JwtSecret);

    key.as_bytes().into()
}
pub fn is_prod() -> bool {
    from_env(EnvKey::DeployEnv).eq("PROD")
}
