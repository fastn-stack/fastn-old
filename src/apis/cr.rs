#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateRequest {
    pub title: String,
    pub description: Option<String>,
    pub cr_number: Option<usize>,
}

#[derive(Debug, serde::Serialize, Default)]
struct CreateResponse {
    number: usize,
    files: std::collections::HashMap<String, Vec<u8>>, // about.ftd
}

pub(crate) async fn create(
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
    // get the number path = <root>/.cr
    // create a directory under path = <root>/-/<cr-num>/
    // create $path/-/about.ftd
    // return response

    let config = fpm::Config::read2(None, false).await?;
    let cr_number = if let Some(cr_number) = req.cr_number {
        cr_number
    } else {
        fpm::cache::create_or_inc(config.root.join(".cr").as_str()).await?
    };
    let about_content = {
        let mut about_content = format!("-- import: fpm\n\n\n-- fpm.cr-about: {}", req.title);
        if let Some(description) = req.description {
            about_content = format!("{}\n\n{}", about_content, description);
        }
        about_content
    };

    fpm::utils::update(
        &config.cr_path(cr_number),
        "-/about.ftd",
        about_content.as_bytes(),
    )
    .await?;

    Ok(CreateResponse {
        number: cr_number,
        files: std::array::IntoIter::new([(
            config.cr_path(cr_number).join("-/about.ftd").to_string(),
            about_content.as_bytes().to_vec(),
        )])
        .collect(),
    })
}
