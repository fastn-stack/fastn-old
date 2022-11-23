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
pub const USER_DETAIL: &str = "ud";
pub const GITHUB_PROVIDER: &str = "github";
//pub const TELEGRAM_PROVIDER: &str = "telegram";
//pub const DISCORD_PROVIDER: &str = "discord";
//pub const SLACK_PROVIDER: &str = "slack";
//pub const GOOGLE_PROVIDER: &str = "google";

// TODO: rename the method later
// bridge between fpm to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use magic_crypt::MagicCryptTrait;
    let encryption_key = std::env::var("ENCRYPT_KEY").expect("ENCRYPT_KEY not set in env");
    let mc_obj = magic_crypt::new_magic_crypt!(encryption_key, 256);

    let mut matched_identities: Vec<UserIdentity> = vec![];

    let ud = cookies.get(USER_DETAIL).ok_or_else(|| {
        fpm::Error::GenericError("user detail not found in the cookies".to_string())
    })?;

    if let Ok(ud_decrypted) = mc_obj.decrypt_base64_to_string(ud) {
        let ud_list: Vec<github::UserDetail> = serde_json::from_str(ud_decrypted.as_str())?;

        let github_ud_opt = ud_list
            .into_iter()
            .find(|ud_obj| ud_obj.provider.eq(GITHUB_PROVIDER));
        if let Some(ud) = github_ud_opt {
            matched_identities.extend(github::matched_identities(ud, identities).await?);
        };
    }
    //dbg!(dec_obj);

    //dbg!(found_obj);
    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    //let matched_identities = github::matched_identities(token.as_str(), identities).await?;

    //TODO: Call discord::matched_identities
    //TODO: Call google::matched_identities
    //TODO: Call twitter::matched_identities
    dbg!(&matched_identities);
    Ok(matched_identities)
}
