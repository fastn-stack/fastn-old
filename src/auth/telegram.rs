// TODO: This has be set while creating the Telegram OAuth Application
pub const ACCESS_URL: &str = "/auth/telegram/access/";
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
}
// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // This method will be called to open telegram login dialogue
    dbg!("telegram login");
    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        ACCESS_URL
    );
    let login_widget_url = "https://telegram.org/js/telegram-widget.js?21";
    //let telegram_bot_title = "fpmapp_bot";

    /*let telegram_body = format!(
        "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
        "<html>
        <head><title>FDM</title></head>
        <body><script async src=",
        r#"""#,
        login_widget_url,
        r#"""#,
        "data-telegram-login=",
        r#"""#,
        telegram_bot_title,
        r#"""#,
        "data-size=",
        r#"""#,
        "large",
        r#"""#,
        " data-auth-url=",
        r#"""#,
        redirect_url,
        r#"""#,
        " data-request-access=",
        r#"""#,
        "write",
        r#"""#,
        ">return TWidgetLogin.auth();</script>"
    );*/
    let telegram_body = format!(
        "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
        "<html>
        <head><title>FTD</title></head>
        <body onload=",
        r#"""#,
        "telegramAuth()",
        r#"""#,
        "><script async src=",
        r#"""#,
        login_widget_url,
        r#"""#,
        r#"></script><script type='text/javascript'>function telegramAuth(){window.Telegram.Login.auth(
            { bot_id: '"#,
        std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set in env"),
        r#"', request_access: true },
            (data) => {
              if (!data) {
                // authorization failed
              }

window.location.replace("#,
        r#"""#,
        redirect_url,
        r#"?id=""#,
        r#"+data.id+"&first_name="+data.first_name+"&last_name="+data.last_name+"&username="+data.username+"&auth_date="+data.auth_date+"&hash="+data.hash);
            }
          );}</script></body>
          </html>"#
    );
    Ok(actix_web::HttpResponse::Ok().body(telegram_body))
}
// route: /auth/telegram/access/
// In this API we are accessing
// the token and setting it to cookies
pub async fn token(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    dbg!("came here");
    use magic_crypt::MagicCryptTrait;
    #[derive(serde::Deserialize)]
    pub struct QueryParams {
        pub id: String,
        pub first_name: String,
        pub last_name: String,
        pub username: String,
        pub auth_date: String,
        pub hash: String,
    }
    let encryption_key = std::env::var("ENCRYPT_KEY").expect("ENCRYPT_KEY not set in env");
    let mc_obj = magic_crypt::new_magic_crypt!(encryption_key, 256);

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    dbg!(&query.hash);
    let token = query.hash;
    let user_name = query.username;
    let user_detail_obj: UserDetail = UserDetail { token, user_name };
    let user_detail_str = serde_json::to_string(&user_detail_obj)?;
    dbg!(&user_detail_str);
    return Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(
                fpm::auth::TELEGRAM_PROVIDER,
                mc_obj
                    .encrypt_to_base64(&user_detail_str)
                    .as_str()
                    .to_owned(),
            )
            .domain(fpm::auth::utils::domain(req.connection_info().host()))
            .path("/")
            .permanent()
            .secure(true)
            .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish());
}
