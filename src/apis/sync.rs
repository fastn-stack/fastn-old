#[derive(serde::Serialize)]
struct ApiResponse {
    name: String,
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

pub(crate) fn sync() -> actix_web::Result<actix_web::HttpResponse> {
    let t = ApiResponse {
        name: "".to_string(),
    };
    success(t)
}
