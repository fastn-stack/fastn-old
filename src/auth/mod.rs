pub(crate) mod amazon;
pub(crate) mod apple;
pub(crate) mod baidu;
pub(crate) mod bitbucket;
pub(crate) mod config;
pub(crate) mod digitalocean;
pub(crate) mod discord;
pub(crate) mod doorkeeper;
pub(crate) mod dropbox;
pub(crate) mod facebook;
pub(crate) mod github;
pub(crate) mod gitlab;
pub(crate) mod gmail;
pub(crate) mod google;
pub(crate) mod instagram;
pub(crate) mod linkedin;
pub(crate) mod microsoft;
pub(crate) mod okta;
pub(crate) mod pintrest;
pub(crate) mod processor;
pub(crate) mod routes;
pub(crate) mod slack;
pub(crate) mod telegram;
pub(crate) mod tiktok;
pub(crate) mod twitch;
pub(crate) mod twitter;
pub(crate) mod wechat;
pub(crate) mod yahoo;
pub(crate) mod zoho;

pub mod utils;

pub(crate) enum AuthProviders {
    GitHub,
    TeleGram,
    Google,
    Discord,
    Slack,
    Amazon,
    Apple,
    Baidu,
    BitBucket,
    DigitalOcean,
    DoorKeeper,
    DropBox,
    Facebook,
    GitLab,
    Instagram,
    LinkedIn,
    Microsoft,
    Okta,
    Pintrest,
    TikTok,
    Twitch,
    Twitter,
    WeChat,
    Yahoo,
    Zoho,
}

impl AuthProviders {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AuthProviders::GitHub => "github",
            AuthProviders::TeleGram => "telegram",
            AuthProviders::Google => "google",
            AuthProviders::Discord => "discord",
            AuthProviders::Slack => "slack",
            AuthProviders::Amazon => "amazon",
            AuthProviders::Apple => "apple",
            AuthProviders::Baidu => "baidu",
            AuthProviders::BitBucket => "bitbucket",
            AuthProviders::DigitalOcean => "digitalocean",
            AuthProviders::DoorKeeper => "doorkeeper",
            AuthProviders::DropBox => "dropbox",
            AuthProviders::Facebook => "facebook",
            AuthProviders::GitLab => "gitlab",
            AuthProviders::Instagram => "instagram",
            AuthProviders::LinkedIn => "linkedin",
            AuthProviders::Microsoft => "microsoft",
            AuthProviders::Okta => "okta",
            AuthProviders::Pintrest => "pintrest",
            AuthProviders::TikTok => "tiktok",
            AuthProviders::Twitch => "twitch",
            AuthProviders::Twitter => "twitter",
            AuthProviders::WeChat => "wechat",
            AuthProviders::Yahoo => "yahoo",
            AuthProviders::Zoho => "zoho",
        }
    }
}

pub fn secret_key() -> String {
    match std::env::var("SECRET_KEY") {
        Ok(secret) => secret,
        Err(_e) => {
            println!("WARN: SECRET_KEY not set");
            // TODO: Need to change this approach later
            "FPM_TEMP_SECRET".to_string()
        }
    }
}

// TODO: rename the method later
// bridge between fpm to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    let mut matched_identities: Vec<fpm::user_group::UserIdentity> = vec![];

    let github_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::GitHub.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("github user detail not found in the cookies".to_string())
        });
    match github_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(github_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let github_ud: github::UserDetail =
                    serde_json::from_str(github_ud_decrypted.as_str())?;
                matched_identities.extend(github::matched_identities(github_ud, identities).await?);
            }
        }
        Err(err) => {
            // TODO: What to do with this error
            dbg!(format!(
                "{}{}",
                "github user detail not found in the cookies", err
            ));
        }
    };
    let telegram_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::TeleGram.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("telegram user detail not found in the cookies".to_string())
        });
    match telegram_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(telegram_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let telegram_ud: telegram::UserDetail =
                    serde_json::from_str(telegram_ud_decrypted.as_str())?;
                matched_identities
                    .extend(telegram::matched_identities(telegram_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "telegram user detail not found in the cookies", err);
        }
    };
    let discord_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Discord.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("discord user detail not found in the cookies".to_string())
        });
    match discord_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(discord_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let discord_ud: discord::UserDetail =
                    serde_json::from_str(discord_ud_decrypted.as_str())?;
                matched_identities
                    .extend(discord::matched_identities(discord_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "discord user detail not found in the cookies", err);
        }
    };
    let twitter_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Twitter.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("twitter user detail not found in the cookies".to_string())
        });
    match twitter_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(twitter_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let twitter_ud: twitter::UserDetail =
                    serde_json::from_str(twitter_ud_decrypted.as_str())?;
                matched_identities
                    .extend(twitter::matched_identities(twitter_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "twitter user detail not found in the cookies", err);
        }
    };
    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    //let matched_identities = github::matched_identities(token.as_str(), identities).await?;

    //TODO: Call discord::matched_identities
    //TODO: Call google::matched_identities
    //TODO: Call twitter::matched_identities
    Ok(matched_identities)
}
