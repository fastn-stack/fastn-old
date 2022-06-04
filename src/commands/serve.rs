use itertools::Itertools;

async fn serve_static(req: actix_web::HttpRequest) -> actix_web::Result<actix_files::NamedFile> {
    // TODO: It should ideally fallback to index file if not found than an error file or directory listing
    // TODO:
    // .build directory should come from config
    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();
    println!("url arg path : {:?}", path);

    let file_path = std::path::PathBuf::new().join(".build").join(path.clone());

    let file_path = if file_path.is_file() {
        file_path
    } else {
        std::path::PathBuf::new().join(file_path).join("index.html")
    };

    println!("static file Path: {:?}", file_path.to_str());

    let mut config = fpm::Config::read(None).await.unwrap();

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

    let all_dep_name = dependencies.iter().map(|d| d.package.name.as_str()).collect_vec();
    println!("ALL Deps name {:?}", all_dep_name);

    let file_name = fpm::config::Config::get_file_name(&config.root, path.to_str().unwrap());
    println!("url to file name: {:?} {:?}", path, file_name);


    let f = match config.get_file_by_id(path.to_str().unwrap(), &config.package).await {
        Ok(f) => f,
        Err(e) => panic!("Error: {}", e)
    };

    println!("File: {:?}", f);

    let package_name = match f {
        fpm::file::File::Ftd(ref d) => d.package_name.as_str(),
        _ => panic!("file not found")
    };

    let package = dependencies.iter()
        .find(|d| d.package.name.eq(package_name))
        .map(|x| &x.package)
        .unwrap_or(&config.package);

    fpm::commands::build::process_file(
        &config,
        package,
        &f,
        None,
        None,
        Default::default(),
        "/",
        false,
        &asset_documents,
        None,
        true
    ).await.expect("Some error");

    let file = actix_files::NamedFile::open_async(file_path).await?;
    Ok(file)
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
