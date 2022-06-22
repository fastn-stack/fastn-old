use itertools::Itertools;

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub enum SyncStatus {
    Conflict,
    NoConflict,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct File {
    path: String,
    content: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct SyncResponse {
    files: Vec<SyncResponseFile>,
    dot_history: Vec<File>,
    latest_ftd: String,
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
#[serde(tag = "action")]
pub enum SyncRequestFile {
    Add { path: String, content: Vec<u8> },
    Update { path: String, content: Vec<u8> },
    Delete { path: String },
}

impl SyncRequestFile {
    fn id(&self) -> String {
        match self {
            SyncRequestFile::Add { path, .. }
            | SyncRequestFile::Update { path, .. }
            | SyncRequestFile::Delete { path } => path.to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SyncRequest {
    pub package_name: String,
    pub files: Vec<SyncRequestFile>,
    pub latest_ftd: String,
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

fn error<T: Into<String>>(
    message: T,
    status: actix_web::http::StatusCode,
) -> actix_web::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize)]
    struct ErrorResponse {
        message: String,
        success: bool,
    }

    let resp = ErrorResponse {
        message: message.into(),
        success: false,
    };

    Ok(actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::json())
        .status(status)
        .body(serde_json::to_string(&resp)?))
}

/// Steps
/// Read latest.ftd and create snapshot version
/// Iterate over Added files, create them and update new version in latest.ftd
/// Iterate over Deleted Files, If version are same remove it from remote otherwise send updated file
/// Iterate over Update Files, get the base file according to client latest.ftd and apply three way merge,
/// If no conflict merge it, update file on remote and send back new content as Updated
/// If conflict occur, Then send back updated version in latest.ftd with conflicted content
///
pub async fn sync(
    req: actix_web::web::Json<SyncRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    return match sync_worker(req.0) {
        Ok(data) => success(data),
        Err(err) => error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    };

    // success(r)
}

pub(crate) fn sync_worker(request: SyncRequest) -> fpm::Result<SyncResponse> {
    dbg!(&request.files.iter().map(|x| x.id()).collect_vec());
    let r = SyncResponse {
        files: vec![],
        dot_history: vec![],
        latest_ftd: "".to_string(),
    };
    Ok(r)
}

// #[derive(Debug, std::fmt::Display)]
// struct ApiResponseError {
//     message: String,
//     success: bool,
// }

// TODO: Fir kabhi
// impl actix_web::ResponseError for ApiResponseError {}
