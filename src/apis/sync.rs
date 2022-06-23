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
    #[derive(serde::Serialize, Debug)]
    struct ErrorResponse {
        message: String,
        success: bool,
    }

    let resp = ErrorResponse {
        message: message.into(),
        success: false,
    };

    dbg!(&resp);

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
    return match sync_worker(req.0).await {
        Ok(data) => success(data),
        Err(err) => error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    };

    // success(r)
}

pub(crate) async fn sync_worker(request: SyncRequest) -> fpm::Result<SyncResponse> {
    dbg!(&request.files.iter().map(|x| x.id()).collect_vec());
    // TODO: Need to call at once only
    let config = fpm::Config::read2(None, false).await?;
    dbg!("config read success");
    let mut snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
    dbg!("get latest snapshot");
    let client_snapshots = fpm::snapshot::resolve_snapshots(&request.latest_ftd).await?;
    dbg!("get client snapshot");
    // let latest_ftd = tokio::fs::read_to_string(config.history_dir().join(".latest.ftd")).await?;
    let timestamp = fpm::timestamp_nanosecond();
    let mut synced_files = vec![];
    let mut dot_history = vec![];

    dbg!("started loop");
    for file in request.files.iter() {
        match file {
            SyncRequestFile::Add { path, content } => {
                dbg!("ADD", &file.id());
                fpm::utils::write(&config.root, path, content).await?;

                dbg!(&config.root.join(path).as_str());

                let snapshot_path =
                    fpm::utils::history_path(path, config.root.as_str(), &timestamp);

                dbg!(&snapshot_path);

                if let Some((dir, _)) = snapshot_path.as_str().rsplit_once('/') {
                    tokio::fs::create_dir_all(dir).await?;
                }

                tokio::fs::copy(config.root.join(path), snapshot_path).await?;
                dbg!("Copy Success to history");
                snapshots.insert(path.to_string(), timestamp);
                dot_history.push(File {
                    path: fpm::utils::snapshot_id(path, &timestamp),
                    content: content.to_vec(),
                });
                // Create a new file
                // Take snapshot
                // Update version in latest.ftd
            }
            SyncRequestFile::Delete { path } => {
                dbg!("Delete", &file.id());
                if config.root.join(path).exists() {
                    tokio::fs::remove_file(config.root.join(path)).await?;
                }
                snapshots.remove(path);
            }
            SyncRequestFile::Update { path, content } => {
                dbg!("Update", &file.id());
                let client_snapshot_timestamp =
                    client_snapshots
                        .get(path)
                        .ok_or(fpm::Error::APIResponseError(format!(
                            "path should be available in latest.ftd {}",
                            path
                        )))?;
                dbg!("Update", &path, &client_snapshot_timestamp);
                // TODO: It may have been deleted
                let snapshot_timestamp =
                    snapshots
                        .get(path)
                        .ok_or(fpm::Error::APIResponseError(format!(
                            "path should be available in latest.ftd {}",
                            path
                        )))?;
                dbg!("Update", &path, &snapshot_timestamp);
                // No conflict case
                if client_snapshot_timestamp.eq(snapshot_timestamp) {
                    dbg!("Both version are equal");
                    fpm::utils::update(&config.root, path, content).await?;
                    let snapshot_path =
                        fpm::utils::history_path(path, config.root.as_str(), &timestamp);
                    tokio::fs::copy(config.root.join(path), snapshot_path).await?;
                    snapshots.insert(path.to_string(), timestamp);
                    dot_history.push(File {
                        path: fpm::utils::snapshot_id(path, &timestamp),
                        content: content.to_vec(),
                    });
                } else {
                    dbg!("Both version are not equal");
                    // TODO: Need to handle static files like images, don't require merging
                    let ancestor_path = fpm::utils::history_path(
                        path,
                        config.root.as_str(),
                        &client_snapshot_timestamp,
                    );
                    let ancestor_content = tokio::fs::read_to_string(ancestor_path).await?;
                    let ours_path =
                        fpm::utils::history_path(path, config.root.as_str(), &snapshot_timestamp);
                    let ours_content = tokio::fs::read_to_string(ours_path).await?;
                    let theirs_content = String::from_utf8(content.clone())
                        .map_err(|e| fpm::Error::APIResponseError(e.to_string()))?;

                    match diffy::merge(&ancestor_content, &ours_content, &theirs_content) {
                        Ok(data) => {
                            fpm::utils::update(&config.root, path, data.as_bytes()).await?;
                            let snapshot_path =
                                fpm::utils::history_path(path, config.root.as_str(), &timestamp);
                            tokio::fs::copy(config.root.join(path), snapshot_path).await?;
                            snapshots.insert(path.to_string(), timestamp);
                            dot_history.push(File {
                                path: fpm::utils::snapshot_id(path, &timestamp),
                                content: content.to_vec(),
                            });
                        }
                        Err(data) => {
                            // Return conflicted content
                            synced_files.push(SyncResponseFile::Update {
                                path: path.to_string(),
                                status: SyncStatus::Conflict,
                                content: data.as_bytes().to_vec(),
                            })
                        }
                    }
                }
            }
        }
    }

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
