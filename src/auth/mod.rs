use crate::user_group::UserIdentity;

pub(crate) mod config;
pub(crate) mod discord;
pub(crate) mod github;
pub(crate) mod gmail;
pub(crate) mod processor;
pub(crate) mod routes;
pub(crate) mod slack;
pub(crate) mod telegram;
pub mod utils;

pub const COOKIE_TOKEN: &str = "token";
pub const GITHUB_PROVIDER: &str = "github";
pub const TELEGRAM_PROVIDER: &str = "telegram";
pub const DISCORD_PROVIDER: &str = "discord";
pub const SLACK_PROVIDER: &str = "slack";
pub const GOOGLE_PROVIDER: &str = "google";

// TODO: rename the method later
// bridge between fpm to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use magic_crypt::MagicCryptTrait;
    let secret_key = match std::env::var("SECRET_KEY") {
        Ok(val) => val,
        Err(e) => format!("{}{}", "SECRET_KEY not set in env ", e),
    };
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key, 256);

    let mut matched_identities: Vec<UserIdentity> = vec![];

    let github_ud_encrypted = cookies.get(fpm::auth::GITHUB_PROVIDER).ok_or_else(|| {
        fpm::Error::GenericError("user detail not found in the cookies".to_string())
    })?;
    if let Ok(github_ud_decrypted) = mc_obj.decrypt_base64_to_string(github_ud_encrypted) {
        let github_ud: github::UserDetail = serde_json::from_str(github_ud_decrypted.as_str())?;
        matched_identities.extend(github::matched_identities(github_ud, identities).await?);
    }

    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    //let matched_identities = github::matched_identities(token.as_str(), identities).await?;

    //TODO: Call discord::matched_identities
    //TODO: Call google::matched_identities
    //TODO: Call twitter::matched_identities
    Ok(matched_identities)
}
