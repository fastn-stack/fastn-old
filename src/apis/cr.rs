#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub cr_number: Option<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct CreateResponse {
    pub number: usize,
    pub files: std::collections::HashMap<String, Vec<u8>>, // about.ftd
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
    let cr_about = if let Some(cr_number) = req.cr_number {
        let mut cr_about = fpm::cr::get_cr_about(&config, cr_number).await?;
        if let Some(title) = req.title {
            cr_about.title = title;
        }
        if let Some(description) = req.description {
            cr_about.description = Some(description);
        }
        cr_about
    } else {
        let cr_number = fpm::cache::create_or_inc(config.root.join(".cr").as_str()).await?;
        fpm::cr::CRAbout {
            title: req.title.unwrap_or_else(|| cr_number.to_string()),
            description: req.description,
            cr_number,
        }
    };
    fpm::cr::create_cr_about(&config, &cr_about).await?;

    Ok(CreateResponse {
        number: cr_about.cr_number,
        files: std::array::IntoIter::new([(
            format!("-/{}/-/about.ftd", cr_about.cr_number),
            fpm::cr::generate_cr_about_content(&cr_about)
                .as_bytes()
                .to_vec(),
        )])
        .collect(),
    })
}

pub(crate) async fn client_create(
    req: actix_web::web::Json<CreateRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    match client_create_(req.0).await {
        Ok(cr_number) => {
            #[derive(serde::Serialize)]
            struct CreateCRResponse {
                url: String,
            }
            fpm::apis::success(CreateCRResponse {
                url: format!("-/view-src/-/{}/-/about/", cr_number),
            })
        }
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

async fn client_create_(req: CreateRequest) -> fpm::Result<usize> {
    let config = fpm::Config::read2(None, false).await?;
    let response = fpm::commands::create_cr::cr(req.title, req.description, req.cr_number).await?;
    for (file_path, content) in response.files {
        fpm::utils::update(&config.root, file_path.as_str(), content.as_slice()).await?;
    }
    Ok(response.number)
}
