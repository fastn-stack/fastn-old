pub(crate) mod config;
pub(crate) mod discord;
pub(crate) mod github;
pub(crate) mod gmail;
pub(crate) mod processor;
pub(crate) mod routes;
pub(crate) mod slack;
pub(crate) mod telegram;
pub mod utils;

pub(crate) enum AuthProviders {
    GitHub,
    TeleGram,
    Google,
    Discord,
    Slack,
}

impl AuthProviders {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AuthProviders::GitHub => "github",
            AuthProviders::TeleGram => "telegram",
            AuthProviders::Google => "google",
            AuthProviders::Discord => "discord",
            AuthProviders::Slack => "slack",
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

/// will fetch out the decrypted github user data from cookies
/// and return it as string
/// if no github cookie found then it returns None
pub async fn get_github_ud_from_cookies(
    cookies: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let github_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::GitHub.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("github user detail not found in the cookies".to_string())
        });
    match github_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(github_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let github_ud: github::UserDetail =
                    serde_json::from_str(github_ud_decrypted.as_str()).ok()?;
                let ud_string = format!("{}-{}", &github_ud.user_name, &github_ud.token);
                return Some(ud_string);
            }
        }
        Err(err) => {
            // Debug out the error and return None
            let error_msg = format!("GITHUB UD error: {}", err);
            dbg!(&error_msg);
        }
    };
    None
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
    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    //let matched_identities = github::matched_identities(token.as_str(), identities).await?;

    //TODO: Call discord::matched_identities
    //TODO: Call google::matched_identities
    //TODO: Call twitter::matched_identities
    Ok(matched_identities)
}
