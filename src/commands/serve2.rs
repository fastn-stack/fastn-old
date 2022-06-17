async fn handle_ftd(config: &mut fpm::Config, path: std::path::PathBuf) -> actix_web::HttpResponse {
    let path = match path.to_str() {
        Some(s) => s,
        None => {
            println!("handle_ftd: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    let dep_package = if let Ok(dep) = config
        .resolve_package(find_dep_package(config, &config.package, path))
        .await
    {
        dep
    } else {
        println!("handle_ftd: Cannot resolve package");
        return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
    };

    config.add_package(&dep_package);

    let f = match config.get_file_by_id2(path, &dep_package).await {
        Ok(f) => f,
        Err(e) => {
            println!("new_path: {}, Error: {:?}", path, e);
            return actix_web::HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    config.current_document = Some(f.get_id());
    return match f {
        fpm::File::Ftd(main_document) => {
            return match fpm::commands::build::process_ftd(
                config,
                &main_document,
                None,
                None,
                Default::default(),
                "/",
                &Default::default(),
                false,
            )
            .await
            {
                Ok(r) => actix_web::HttpResponse::Ok().body(r),
                Err(e) => actix_web::HttpResponse::InternalServerError().body(e.to_string()),
            };
        }
        _ => actix_web::HttpResponse::InternalServerError().body("".as_bytes()),
    };

    fn find_dep_package<'a>(
        config: &'a fpm::Config,
        package: &'a fpm::Package,
        file_path: &'a str,
    ) -> &'a fpm::Package {
        let file_path = if let Some(file_path) = file_path.strip_prefix("-/") {
            file_path
        } else {
            return &config.package;
        };
        package
            .dependencies
            .iter()
            .find(|d| file_path.starts_with(&d.package.name))
            .map(|x| &x.package)
            .unwrap_or(&config.package)
    }
}

// async fn handle_dash(
//     req: &actix_web::HttpRequest,
//     config: &fpm::Config,
//     path: std::path::PathBuf,
// ) -> actix_web::HttpResponse {
//     let new_path = match path.to_str() {
//         Some(s) => s.replace("-/", ""),
//         None => {
//             println!("handle_dash: Not able to convert path");
//             return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
//         }
//     };
//
//     let file_path = if new_path.starts_with(&config.package.name) {
//         std::path::PathBuf::new().join(
//             new_path
//                 .strip_prefix(&(config.package.name.to_string() + "/"))
//                 .unwrap(),
//         )
//     } else {
//         std::path::PathBuf::new().join(".packages").join(new_path)
//     };
//
//     server_static_file(req, file_path).await
// }

async fn server_static_file(
    req: &actix_web::HttpRequest,
    file_path: std::path::PathBuf,
) -> actix_web::HttpResponse {
    if !file_path.exists() {
        return actix_web::HttpResponse::NotFound().body("".as_bytes());
    }

    match actix_files::NamedFile::open_async(file_path).await {
        Ok(r) => r.into_response(req),
        Err(_e) => actix_web::HttpResponse::NotFound().body("TODO".as_bytes()),
    }
}
async fn serve_static(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let mut config = fpm::Config::read2(None).await.unwrap();
    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();

    let favicon = std::path::PathBuf::new().join("favicon.ico");
    /*if path.starts_with("-/") {
        handle_dash(&req, &config, path).await
    } else*/
    if path.eq(&favicon) {
        server_static_file(&req, favicon).await
    } else if path.eq(&std::path::PathBuf::new().join("")) {
        handle_ftd(&mut config, path.join("index")).await
    } else {
        handle_ftd(&mut config, path).await
    }
}

#[actix_web::main]
pub async fn serve2(bind_address: &str, port: &str) -> std::io::Result<()> {
    if cfg!(feature = "controller") {
        // fpm-controller base path and ec2 instance id (hardcoded for now)
        let fpm_controller: String = std::env::var("FPM_CONTROLLER")
            .unwrap_or_else(|_| "https://controller.fifthtry.com".to_string());
        let fpm_instance: String =
            std::env::var("FPM_INSTANCE_ID").expect("FPM_INSTANCE_ID is required");

        match crate::controller::resolve_dependencies(fpm_instance, fpm_controller).await {
            Ok(_) => println!("Dependencies resolved"),
            Err(e) => panic!("Error resolving dependencies using controller!!: {:?}", e),
        }
    }

    println!("### Server Started ###");
    println!("Go to: http://{}:{}", bind_address, port);
    actix_web::HttpServer::new(|| {
        actix_web::App::new().route("/{path:.*}", actix_web::web::get().to(serve_static))
    })
    .bind(format!("{}:{}", bind_address, port))?
    .run()
    .await
}
