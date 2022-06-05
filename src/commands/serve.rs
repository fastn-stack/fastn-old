use std::io::Read;
use itertools::Itertools;
// actix_web::Result<actix_files::NamedFile>
async fn serve_static(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    // TODO: It should ideally fallback to index file if not found than an error file or directory listing
    // TODO:
    // .build directory should come from config
    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();
    println!("url arg path : {:?}", path);

    //TODO: This is going to be remove
    let file_path = std::path::PathBuf::new().join(".build").join(path.clone());

    let file_path = if file_path.is_file() {
        file_path
    } else {
        std::path::PathBuf::new().join(file_path).join("index.html")
    };

    // println!("static file Path: {:?}", file_path.to_str());

    let config = fpm::Config::read(None).await.unwrap();
    println!("root: {}", config.root);

    let dependencies = if let Some(package) = config.package.translation_of.as_ref() {
        let mut deps = package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec();
        deps.extend(
            config
                .package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone()),
        );
        deps
    } else {
        config
            .package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec()
    };

    let mut asset_documents = std::collections::HashMap::new();
    asset_documents.insert(
        config.package.name.clone(),
        config.package.get_assets_doc(&config, "/").await.unwrap(),
    );

    for dep in &dependencies {
        asset_documents.insert(
            dep.package.name.clone(),
            dep.package.get_assets_doc(&config, "/").await.unwrap(),
        );
    }

    // let all_dep_name = dependencies.iter().map(|d| d.package.name.as_str()).collect_vec();
    // println!("ALL Deps name {:?}", all_dep_name);


    fn find_dep_package<'a>(config: &'a fpm::Config, dep: &'a Vec<fpm::Dependency>, file_path: &'a str) -> &'a fpm::Package {
        dep.iter()
            .find(|d| file_path.starts_with(&d.package.name))
            .map(|x| &x.package).unwrap_or(&config.package)
    }

    // replace -/ from path string
    // if starts with -/ serve it from packages

    if !path.starts_with("-/") {
        let new_path = match path.to_str() {
            Some(s) => s.replace("-/", ""),
            None => panic!("Not able to convert path")
        };

        let dep_package = find_dep_package(&config, &dependencies, &new_path);

        println!("file path {}, dep_package: {}", new_path, dep_package.name);

        let f = match config.get_file_by_id(&new_path, dep_package).await {
            Ok(f) => f,
            Err(e) => panic!("path: {}, Error: {}", new_path, e)
        };

        println!("File found: Id {:?}", f.get_id());

        match f {
            fpm::File::Ftd(main_document) => {
                return match fpm::commands::build::process_ftd(
                    &config,
                    &main_document,
                    None,
                    None,
                    Default::default(),
                    "/",
                    &asset_documents,
                    false
                ).await {
                    Ok(r) => actix_web::HttpResponse::Ok().body(r),
                    Err(e) => actix_web::HttpResponse::InternalServerError().body("".as_bytes())
                };
            }
            _ => ()
        };

    }


    // fpm::commands::build::process_file(
    //     &config,
    //     dep_package,
    //     &f,
    //     None,
    //     None,
    //     Default::default(),
    //     "/",
    //     false,
    //     &asset_documents,
    //     None,
    //     true
    // ).await.expect("Some error");

    let mut buffer = String::new();
    match actix_files::NamedFile::open_async(file_path).await {
        Ok(file) => {
            file.file().read_to_string(&mut buffer).expect("");
            actix_web::HttpResponse::Ok().body(buffer)
             },
        Err(e) => actix_web::HttpResponse::InternalServerError().body("".as_bytes()),
    }
    // Ok(file)
}

#[actix_web::main]
pub async fn serve(port: &str) -> std::io::Result<()> {
    println!("### Server Started ###");
    println!("Go to: http://127.0.0.1:{}", port);
    actix_web::HttpServer::new(|| {
        actix_web::App::new().route("/{path:.*}", actix_web::web::get().to(serve_static))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
