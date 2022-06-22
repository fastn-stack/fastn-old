#[derive(serde::Serialize, std::fmt::Debug)]
pub enum SyncStatus {
    Conflict,
    NoConflict,
}

#[derive(serde::Serialize, std::fmt::Debug)]
#[serde(tag = "action")]
pub enum SyncResponseFile {
    Add {
        path: String,
        status: SyncStatus,
        content: Vec<u8>,
    },
    Update {
        path: String,
        status: SyncStatus,
        content: Vec<u8>,
    },
    Delete {
        path: String,
        status: SyncStatus,
        content: Vec<u8>,
    },
}

#[derive(serde::Serialize, std::fmt::Debug)]
pub struct File {
    path: String,
    content: Vec<u8>,
}

#[derive(serde::Serialize, std::fmt::Debug)]
struct SyncResponse {
    files: Vec<SyncResponseFile>,
    dot_history: Vec<File>,
    latest_ftd: File,
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(tag = "action")]
pub enum SyncRequestFile {
    Add { path: String, content: Vec<u8> },
    Update { path: String, content: Vec<u8> },
    Delete { path: String, content: Vec<u8> },
}

#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct SyncRequest {
    package_name: String,
    files: Vec<SyncRequestFile>,
    latest_ftd: String,
}

fn success(data: impl serde::Serialize) -> actix_web::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize)]
    struct SuccessResponse<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let data = serde_json::to_string(&SuccessResponse {
        data,
        success: true,
    })?;

    Ok(actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::json())
        .status(actix_web::http::StatusCode::OK)
        .body(data))
}

fn error(
    message: &str,
    status: actix_web::http::StatusCode,
) -> actix_web::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize)]
    struct ErrorResponse {
        message: String,
        success: bool,
    }

    let resp = ErrorResponse {
        message: message.to_string(),
        success: false,
    };

    Ok(actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::json())
        .status(status)
        .body(serde_json::to_string(&resp)?))
}

pub async fn sync(
    files: actix_web::web::Json<SyncRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    println!("{:?}", files);
    success("")
}

// #[derive(Debug, std::fmt::Display)]
// struct ApiResponseError {
//     message: String,
//     success: bool,
// }

// TODO: Fir kabhi
// impl actix_web::ResponseError for ApiResponseError {}
