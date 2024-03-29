use std::path::PathBuf;

use super::constants;

pub enum EnvKey {
    AppName,
    JwtSecret,
    JwtExpiresInHours,
    MarpApiRoot,
    PublicFilesRoot,
    FilesRoot,
    RootUser,
    DeployEnv,
    SneuUiRoot,
}

pub fn from_env(env_key: EnvKey) -> &'static str {
    match env_key {
        EnvKey::AppName => dotenv!("APP_NAME"),
        EnvKey::JwtSecret => dotenv!("JWT_SECRET"),
        EnvKey::JwtExpiresInHours => {
            let from_env: &str = dotenv!("JWT_EXPIRES_IN_HOURS");

            if from_env.is_empty() {
                constants::DEFAULT_JWT_EXPIRES_IN
            } else {
                from_env
            }
        }
        EnvKey::MarpApiRoot => dotenv!("MARP_API_ROOT"),
        EnvKey::PublicFilesRoot => dotenv!("PUBLIC_FILES_ROOT"),
        EnvKey::FilesRoot => dotenv!("FILES_ROOT"),
        EnvKey::RootUser => dotenv!("ROOT_USER"),
        EnvKey::DeployEnv => dotenv!("DEPLOY_ENV"),
        EnvKey::SneuUiRoot => dotenv!("SNEU_UI_ROOT"),
    }
}

pub fn public_files_root() -> PathBuf {
    from_env(EnvKey::PublicFilesRoot).into()
}
pub fn files_root() -> PathBuf {
    from_env(EnvKey::FilesRoot).into()
}
pub fn jwt_secret() -> Vec<u8> {
    let key = from_env(EnvKey::JwtSecret);

    key.as_bytes().into()
}
pub fn is_prod() -> bool {
    from_env(EnvKey::DeployEnv).eq("PROD")
}
