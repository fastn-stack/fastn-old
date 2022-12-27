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

    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "github" => AuthProviders::GitHub,
            "telegram" => AuthProviders::TeleGram,
            "google" => AuthProviders::Google,
            "discord" => AuthProviders::Discord,
            "slack" => AuthProviders::Slack,
            _ => panic!("Invalid auth provider {}", s),
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

/// will fetch out the decrypted user data from cookies
/// and return it as string
/// if no cookie wrt to platform found it throws an error
pub async fn get_user_data_from_cookies(
    platform: &str,
    requested_field: &str,
    cookies: &std::collections::HashMap<String, String>,
) -> fpm::Result<Option<String>> {
    let ud_encrypted = cookies.get(platform).ok_or_else(|| {
        fpm::Error::GenericError(format!(
            "user detail not found for platform {} in the cookies",
            platform
        ))
    });
    match ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                match fpm::auth::AuthProviders::from_str(platform) {
                    fpm::auth::AuthProviders::GitHub => {
                        let github_ud: github::UserDetail =
                            serde_json::from_str(ud_decrypted.as_str())?;
                        return match requested_field {
                            "username" | "user_name" | "user-name" => {
                                Ok(Some(github_ud.user_name.clone()))
                            }
                            "token" => Ok(Some(github_ud.token.clone())),
                            _ => Err(fpm::Error::GenericError(format!(
                                "invalid field {} requested for platform {}",
                                requested_field, platform
                            ))),
                        };
                    }
                    fpm::auth::AuthProviders::TeleGram => {
                        let telegram_ud: telegram::UserDetail =
                            serde_json::from_str(ud_decrypted.as_str())?;
                        return match requested_field {
                            "username" | "user_name" | "user-name" => {
                                Ok(Some(telegram_ud.user_name.clone()))
                            }
                            "uid" | "userid" | "user-id" => Ok(Some(telegram_ud.user_id.clone())),
                            "token" => Ok(Some(telegram_ud.token.clone())),
                            _ => Err(fpm::Error::GenericError(format!(
                                "invalid field {} requested for platform {}",
                                requested_field, platform
                            ))),
                        };
                    }
                    fpm::auth::AuthProviders::Discord => {
                        let discord_ud: discord::UserDetail =
                            serde_json::from_str(ud_decrypted.as_str())?;
                        return match requested_field {
                            "username" | "user_name" | "user-name" => {
                                Ok(Some(discord_ud.user_name.clone()))
                            }
                            "id" | "userid" | "user-id" => Ok(Some(discord_ud.user_id.clone())),
                            "token" => Ok(Some(discord_ud.token.clone())),
                            _ => Err(fpm::Error::GenericError(format!(
                                "invalid field {} requested for platform {}",
                                requested_field, platform
                            ))),
                        };
                    }
                    _ => unimplemented!()
                }
            }
        }
        Err(err) => {
            // Debug out the error and return None
            let error_msg = format!("User data error: {}", err);
            dbg!(&error_msg);
        }
    };
    Ok(None)
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
