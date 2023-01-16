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

pub fn from_env(env_key: EnvKey) -> String {
    use dotenv::var;

    match env_key {
        EnvKey::AppName => var("APP_NAME").unwrap_or("Snêu".into()),
        EnvKey::JwtSecret => var("JWT_SECRET").expect("JWT_SECRET must be defined"),
        EnvKey::JwtExpiresInHours => {
            let from_env = var("JWT_EXPRIES_IN_HOURS").unwrap_or("24".into());

            if from_env.is_empty() {
                constants::DEFAULT_JWT_EXPIRES_IN.to_string()
            } else {
                from_env
            }
        }
        EnvKey::MarpApiRoot => var("MARP_API_ROOT").unwrap_or_default(),
        EnvKey::AstraUri => var("ASTRA_URI").expect("ASTRA_URI must be defined"),
        EnvKey::AstraBearerToken => var("ASTRA_BEARER_TOKEN").expect("ASTRA_BEARER_TOKEN must be defined"),
        EnvKey::AstraKeySpace => var("ASTRA_KEY_SPACE").expect("ASTRA_KEY_SPACE must be defined"),
        EnvKey::PublicFilesRoot => var("PUBLIC_FILES_ROOT").unwrap_or("static".into()),
        EnvKey::FilesRoot => var("FILES_ROOT").unwrap_or("static".into()),
        EnvKey::RootUser => var("ROOT_USER").unwrap_or("hbp".into()),
        EnvKey::DeployEnv => var("DEPLOY_ENV").unwrap_or("DEV".into()),
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
