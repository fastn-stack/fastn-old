// -/1/

#[derive(Debug, serde::Deserialize)]
struct CreateRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Serialize, Default)]
struct CreateResponse {
    number: usize,
    files: std::collections::HashMap<String, Vec<u8>>, // about.ftd
}
async fn create(
    req: actix_web::web::Json<CreateRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    match handle_create(req.0).await {
        Ok(data) => fpm::apis::success(data),
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

async fn handle_create(req: CreateRequest) -> fpm::Result<CreateResponse> {
    // get the number
    // create a directory under path = <root>/-/<cr-num>/
    // create $path/-/about.ftd
    // return response

    let config = fpm::Config::read2(None, false).await?;

    Ok(Default::default())
}
