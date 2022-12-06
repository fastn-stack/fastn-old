// TODO: This has be set while creating the Discord OAuth Application
pub const CALLBACK_URL: &str = "/auth/discord/callback/";
pub const AUTH_URL: &str = "https://discord.com/oauth2/authorize";
pub const TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DiscordAuthReq {
    pub client_secret: String,
    pub client_id: String,
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
}
// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // Discord will be redirect to this url after login process completed

    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let client_id = match std::env::var("DISCROD_CLIENT_ID") {
        Ok(id) => id,
        Err(_e) => {
            println!("WARN: DISCROD_CLIENT_ID not set");
            // TODO: Need to change this approach later
            "FPM_TEMP_DISCROD_CLIENT_ID".to_string()
        }
    };
    let generated_link = format!(
        "{}{}{}{}{}{}",
        AUTH_URL,
        "?client_id=",
        client_id,
        "&redirect_uri=",
        redirect_url,
        "&response_type=code&scope=identify%20guilds%20guilds.members.read"
    );
    // send redirect to /auth/discord/callback/
    Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, generated_link))
        .finish())
}

// route: /auth/discord/callback/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    use magic_crypt::MagicCryptTrait;
    #[derive(Debug, serde::Deserialize)]
    pub struct QueryParams {
        pub code: String,
    }

    let secret_key = fpm::auth::secret_key();
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key.as_str(), 256);

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    let redirect_url = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let discord_auth =
        apis::discord_token_api(TOKEN_URL, redirect_url.as_str(), query.code.as_str()).await;
    match discord_auth {
        Ok(access_token) => {
            let user_name = apis::user_details(&access_token).await?;
            let user_detail_obj: UserDetail = UserDetail {
                token: access_token.to_owned(),
                user_name,
            };
            let user_detail_str = serde_json::to_string(&user_detail_obj)?;

            return Ok(actix_web::HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build(
                        fpm::auth::AuthProviders::Discord.as_str(),
                        mc_obj
                            .encrypt_to_base64(&user_detail_str)
                            .as_str()
                            .to_owned(),
                    )
                    .domain(fpm::auth::utils::domain(req.connection_info().host()))
                    .path("/")
                    .permanent()
                    .finish(),
                )
                .append_header((actix_web::http::header::LOCATION, "/".to_string()))
                .finish());
        }
        Err(err) => Ok(actix_web::HttpResponse::InternalServerError().body(err.to_string())),
    }
}
// it returns identities which matches to given input
pub async fn matched_identities(
    _ud: UserDetail,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    let discord_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("discord"))
        .collect::<Vec<&fpm::user_group::UserIdentity>>();

    if discord_identities.is_empty() {
        return Ok(vec![]);
    }

    let matched_identities = vec![];

    Ok(matched_identities)
}

pub mod apis {
    #[derive(Debug, serde::Deserialize)]
    pub struct DiscordAuthResp {
        pub access_token: String,
    }
    // TODO: API to starred a repo on behalf of the user
    // API Docs: https://discord.com/developers/docs/getting-started
    //API EndPoints: https://github.com/GregTCLTK/Discord-Api-Endpoints/blob/master/Endpoints.md
    // TODO: It can be stored in the request cookies
    pub async fn user_details(token: &str) -> fpm::Result<String> {
        // API Docs: https://discord.com/api/users/@me
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserDetails {
            username: String,
        }
        let user_obj: UserDetails = get_api("https://discord.com/api/users/@me", token).await?;

        Ok(String::from(&user_obj.username))
    }
    pub async fn get_api<T: serde::de::DeserializeOwned>(url: &str, token: &str) -> fpm::Result<T> {
        let response = reqwest::Client::new()
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("{}{}", "Bearer ", token),
            )
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("fpm"),
            )
            .send()
            .await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fpm::Error::APIResponseError(format!(
                "DISCORD-API-ERROR: {}, Error: {}",
                url,
                response.text().await?
            )));
        }

        Ok(response.json().await?)
    }
    //This API will only be used to get access token for discord
    pub async fn discord_token_api(
        url: &str,
        redirect_url: &str,
        code: &str,
    ) -> fpm::Result<String> {
        let client_id = match std::env::var("DISCROD_CLIENT_ID") {
            Ok(id) => id,
            Err(_e) => {
                println!("WARN: DISCROD_CLIENT_ID not set");
                // TODO: Need to change this approach later
                "FPM_TEMP_DISCROD_CLIENT_ID".to_string()
            }
        };
        let client_secret = match std::env::var("DISCORD_CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(_e) => {
                println!("WARN: DISCORD_CLIENT_SECRET not set");
                // TODO: Need to change this approach later
                "FPM_TEMP_DISCORD_CLIENT_SECRET".to_string()
            }
        };
        let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
        map.insert("client_secret", client_secret.as_str());
        map.insert("client_id", client_id.as_str());
        map.insert("grant_type", "authorization_code");
        map.insert("code", code);
        map.insert("redirect_uri", redirect_url);

        let response = reqwest::Client::new().post(url).form(&map).send().await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fpm::Error::APIResponseError(format!(
                "DISCORD-API-ERROR: {}, Error: {}",
                url,
                response.text().await?
            )));
        }
        let auth_obj = response.json::<DiscordAuthResp>().await?;
        Ok(auth_obj.access_token)
    }
}
