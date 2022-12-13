// Return the login information of the user
pub fn user_details<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let is_login = match &config.request {
        Some(req) => {
            req.cookie(fpm::auth::AuthProviders::GitHub.as_str())
                .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::TeleGram.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Discord.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Slack.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Google.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Amazon.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Apple.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Baidu.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::BitBucket.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::DigitalOcean.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::DoorKeeper.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::DropBox.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Facebook.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::GitLab.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Instagram.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::LinkedIn.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Microsoft.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Okta.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Pintrest.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::TikTok.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Twitch.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Twitter.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::WeChat.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Yahoo.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Zoho.as_str())
                    .is_some()
        }
        None => false,
    };

    #[derive(Debug, serde::Serialize)]
    struct UserDetails {
        #[serde(rename = "is-login")]
        is_login: bool,
    }
    let ud = UserDetails { is_login };
    doc.from_json(&ud, section)
}
