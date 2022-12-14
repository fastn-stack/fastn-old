// TODO: This has be set while creating the Discord OAuth Application
pub const CALLBACK_URL: &str = "/auth/twitter/callback/";
pub const AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";
pub const TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
    pub user_id: String,
}

pub(crate) enum TwitterScopes {
    ReadTweet,
    WriteTweet,
    ModerateTweet,
    ReadUsers,
    ReadFollows,
    WriteFollows,
    AccessOffline,
    ReadSpace,
    ReadMute,
    WriteMute,
    ReadLike,
    WriteLike,
    ReadBlock,
    WriteBlock,
    ReadBookmark,
    WriteBookmark,
    WriteList,
    ReadList,
}

impl TwitterScopes {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TwitterScopes::ReadTweet => "tweet.read",
            TwitterScopes::WriteTweet => "tweet.write",
            TwitterScopes::ModerateTweet => "tweet.moderate.write",
            TwitterScopes::ReadUsers => "users.read",
            TwitterScopes::ReadFollows => "follows.read",
            TwitterScopes::WriteFollows => "follows.write",
            TwitterScopes::AccessOffline => "offline.access",
            TwitterScopes::ReadSpace => "space.read",
            TwitterScopes::ReadMute => "mute.read",
            TwitterScopes::WriteMute => "mute.write",
            TwitterScopes::ReadLike => "like.read",
            TwitterScopes::WriteLike => "like.write",
            TwitterScopes::ReadBlock => "block.read",
            TwitterScopes::WriteBlock => "block.write",
            TwitterScopes::ReadBookmark => "bookmark.read",
            TwitterScopes::WriteBookmark => "bookmark.write",
            TwitterScopes::WriteList => "list.read",
            TwitterScopes::ReadList => "list.write",
        }
    }
}
// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // Twitter will be redirect to this url after login process completed

    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let client_id = match std::env::var("TWITTER_CLIENT_ID") {
        Ok(id) => id,
        Err(_e) => {
            return Err(fpm::Error::APIResponseError(
                "WARN: FPM_TEMP_TWITTER_CLIENT_ID not set.".to_string(),
            ));
            // TODO: Need to change this approach later
            //"FPM_TEMP_TWITTER_CLIENT_ID".to_string()
        }
    };
    let twitter_auth_url = format!(
        "{}{}{}{}{}{}{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        AUTH_URL,
        "?client_id=",
        client_id,
        "&redirect_uri=",
        redirect_url,
        "&response_type=code&state=state&code_challenge=challenge&code_challenge_method=plain&scope=",
        TwitterScopes::ReadTweet.as_str(),
        TwitterScopes::WriteTweet.as_str(),
        TwitterScopes::ModerateTweet.as_str(),
        TwitterScopes::ReadUsers.as_str(),
        TwitterScopes::ReadFollows.as_str(),
        TwitterScopes::WriteFollows.as_str(),
        TwitterScopes::AccessOffline.as_str(),
        TwitterScopes::ReadSpace.as_str(),
        TwitterScopes::ReadMute.as_str(),
        TwitterScopes::WriteMute.as_str(),
        TwitterScopes::ReadLike.as_str(),
        TwitterScopes::WriteLike.as_str(),
        TwitterScopes::ReadBlock.as_str(),
        TwitterScopes::WriteBlock.as_str(),
        TwitterScopes::ReadBookmark.as_str(),
        TwitterScopes::WriteBookmark.as_str(),
        TwitterScopes::WriteList.as_str(),
        TwitterScopes::ReadList.as_str(),
    );
    // send redirect to /auth/twitter/callback/
    Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, twitter_auth_url))
        .finish())
}
// route: /auth/discord/callback/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    #[derive(Debug, serde::Deserialize)]
    pub struct QueryParams {
        pub code: String,
        pub state: String,
    }

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    let redirect_url = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let twitter_auth =
        apis::twitter_token(TOKEN_URL, redirect_url.as_str(), query.code.as_str()).await;
    match twitter_auth {
        Ok(access_token) => {
            dbg!(&access_token);
            let (user_name, user_id) = apis::user_details(&access_token).await?;
            let user_detail_obj: UserDetail = UserDetail {
                token: access_token.to_owned(),
                user_name,
                user_id,
            };
            let user_detail_str = serde_json::to_string(&user_detail_obj)?;
            dbg!(&user_detail_obj);
            return Ok(actix_web::HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build(
                        fpm::auth::AuthProviders::Twitter.as_str(),
                        fpm::auth::utils::encrypt_str(&user_detail_str).await,
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
pub async fn matched_identities(
    _ud: UserDetail,
    _identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    /*let twitter_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("twitter"))
        .collect::<Vec<&fpm::user_group::UserIdentity>>();

    if twitter_identities.is_empty() {
        return Ok(vec![]);
    }*/

    let matched_identities = vec![];

    Ok(matched_identities)
}
pub mod apis {
    #[derive(serde::Deserialize)]
    pub struct TwitterAuthResp {
        pub access_token: String,
    }
    // TODO: API to get user detail.
    // API Docs: https://developer.twitter.com/en/docs/authentication/guides/v2-authentication-mapping
    // TODO: It can be stored in the request cookies
    pub async fn user_details(token: &str) -> fpm::Result<(String, String)> {
        // API Docs: https://api.twitter.com/2/users/me
        #[derive(serde::Deserialize)]
        struct UserDetails {
            data: UserObj,
        }
        #[derive(serde::Deserialize)]
        struct UserObj {
            username: String,
            id: String,
        }
        let user_obj: UserDetails = fpm::auth::utils::get_api(
            "https://api.twitter.com/2/users/me",
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;

        Ok((user_obj.data.username, user_obj.data.id))
    }

    //This API will only be used to get access token for discord
    pub async fn twitter_token(url: &str, redirect_url: &str, code: &str) -> fpm::Result<String> {
        let client_id = match std::env::var("TWITTER_CLIENT_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: TWITTER_CLIENT_ID not set.".to_string(),
                ));
            }
        };

        let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

        map.insert("client_id", client_id.as_str());

        map.insert("code", code);
        map.insert("redirect_uri", redirect_url);
        map.insert("grant_type", "authorization_code");
        map.insert("code_verifier", "challenge");

        let response = reqwest::Client::new().post(url).form(&map).send().await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fpm::Error::APIResponseError(format!(
                "TWITTER-API-ERROR: {}, Error: {}",
                url,
                response.text().await?
            )));
        }
        let auth_obj = response.json::<TwitterAuthResp>().await?;
        Ok(auth_obj.access_token)
    }
}
