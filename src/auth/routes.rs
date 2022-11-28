pub fn is_login(req: &actix_web::HttpRequest) -> bool {
    req.cookie(fpm::auth::GITHUB_PROVIDER).is_some()
        || req.cookie(fpm::auth::TELEGRAM_PROVIDER).is_some()
        || req.cookie(fpm::auth::DISCORD_PROVIDER).is_some()
        || req.cookie(fpm::auth::SLACK_PROVIDER).is_some()
        || req.cookie(fpm::auth::GOOGLE_PROVIDER).is_some()
}

// route: /auth/login/
pub async fn login(
    req: actix_web::HttpRequest,
    edition: Option<String>,
) -> fpm::Result<actix_web::HttpResponse> {
    if is_login(&req) {
        return Ok(actix_web::HttpResponse::Found()
            .append_header((actix_web::http::header::LOCATION, "/".to_string()))
            .finish());
    }

    #[derive(serde::Deserialize)]
    pub struct QueryParams {
        pub platform: String,
    }
    let query = match actix_web::web::Query::<QueryParams>::from_query(req.query_string()) {
        Ok(q) => q,
        Err(err) => {
            dbg!(err);
            return Ok(actix_web::HttpResponse::BadRequest()
                .body("Please select the platform, by which you want to login"));
        }
    };
    match query.platform.as_str() {
        "github" => fpm::auth::github::login(req).await,
        "telegram" => fpm::auth::telegram::login(req).await,
        // TODO: Remove this after demo
        _ => {
            let mut req = fpm::http::Request::from_actix(req, actix_web::web::Bytes::new());
            req.path = "/sorry/".to_string();
            fpm::commands::serve::serve(req, edition).await
        }
        // "discord" => unreachable!(),
        // _ => unreachable!(),
    }
}

// route: /auth/logout/
pub fn logout(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::GITHUB_PROVIDER, "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::TELEGRAM_PROVIDER, "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::SLACK_PROVIDER, "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::DISCORD_PROVIDER, "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::GOOGLE_PROVIDER, "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish())
}

// handle: if request.url starts with /auth/
pub async fn handle_auth(
    req: actix_web::HttpRequest,
    edition: Option<String>,
) -> fpm::Result<fpm::http::Response> {
    if req.path().eq("/auth/login/") {
        return login(req, edition).await;
    } else if req.path().eq(fpm::auth::github::ACCESS_URL) {
        // this will be called after github OAuth login, to set the token
        // It will redirect user to home after the login
        return fpm::auth::github::token(req).await;
    } else if req.path().eq(fpm::auth::telegram::ACCESS_URL) {
        // this will be called after telegram OAuth login, to set the token
        // It will redirect user to home after the login
        return fpm::auth::telegram::token(req).await;
    } else if req.path().eq("/auth/logout/") {
        return logout(req);
    }
    Ok(actix_web::HttpResponse::new(
        actix_web::http::StatusCode::NOT_FOUND,
    ))
}
