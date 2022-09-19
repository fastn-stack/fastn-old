lazy_static! {
    static ref LOCK: async_lock::RwLock<()> = async_lock::RwLock::new(());
}

async fn serve_file(
    req: &actix_web::HttpRequest,
    config: &mut fpm::Config,
    path: &camino::Utf8Path,
) -> actix_web::HttpResponse {
    let f = match config.get_file_and_package_by_id(path.as_str()).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FPM-Error: path: {}, {:?}", path, e);
            return actix_web::HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    // Auth Stuff
    if !f.is_static() {
        match config.can_read(req, path.as_str()).await {
            Ok(can_read) => {
                if !can_read {
                    return actix_web::HttpResponse::Unauthorized()
                        .body(format!("You are unauthorized to access: {}", path));
                }
            }
            Err(e) => {
                eprintln!("FPM-Error: can_read error: {}, {:?}", path, e);
                return actix_web::HttpResponse::InternalServerError().body(e.to_string());
            }
        }
    }

    config.current_document = Some(f.get_id());
    match f {
        fpm::File::Ftd(main_document) => {
            match fpm::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => actix_web::HttpResponse::Ok().body(r),
                Err(e) => {
                    eprintln!("FPM-Error: path: {}, {:?}", path, e);
                    actix_web::HttpResponse::InternalServerError().body(e.to_string())
                }
            }
        }
        fpm::File::Image(image) => actix_web::HttpResponse::Ok()
            .content_type(guess_mime_type(image.id.as_str()))
            .body(image.content),
        fpm::File::Static(s) => actix_web::HttpResponse::Ok().body(s.content),
        _ => {
            eprintln!("FPM unknown handler");
            actix_web::HttpResponse::InternalServerError().body("".as_bytes())
        }
    }
}

async fn serve_cr_file(
    req: &actix_web::HttpRequest,
    config: &mut fpm::Config,
    path: &camino::Utf8Path,
    cr_number: usize,
) -> actix_web::HttpResponse {
    let _lock = LOCK.read().await;
    serve_cr_file_(req, config, path, cr_number).await
}

async fn serve_cr_file_(
    req: &actix_web::HttpRequest,
    config: &mut fpm::Config,
    path: &camino::Utf8Path,
    cr_number: usize,
) -> actix_web::HttpResponse {
    let f = match config
        .get_file_and_package_by_cr_id(path.as_str(), cr_number)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FPM-Error: path: {}, {:?}", path, e);
            return actix_web::HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    // Auth Stuff
    if !f.is_static() {
        match config.can_read(req, path.as_str()).await {
            Ok(can_read) => {
                if !can_read {
                    return actix_web::HttpResponse::Unauthorized()
                        .body(format!("You are unauthorized to access: {}", path));
                }
            }
            Err(e) => {
                eprintln!("FPM-Error: can_read error: {}, {:?}", path, e);
                return actix_web::HttpResponse::InternalServerError().body(e.to_string());
            }
        }
    }

    config.current_document = Some(f.get_id());
    match f {
        fpm::File::Ftd(main_document) => {
            match fpm::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => actix_web::HttpResponse::Ok().body(r),
                Err(e) => {
                    eprintln!("FPM-Error: path: {}, {:?}", path, e);
                    actix_web::HttpResponse::InternalServerError().body(e.to_string())
                }
            }
        }
        fpm::File::Image(image) => actix_web::HttpResponse::Ok()
            .content_type(guess_mime_type(image.id.as_str()))
            .body(image.content),
        fpm::File::Static(s) => actix_web::HttpResponse::Ok().body(s.content),
        _ => {
            eprintln!("FPM unknown handler");
            actix_web::HttpResponse::InternalServerError().body("".as_bytes())
        }
    }
}

fn guess_mime_type(path: &str) -> mime_guess::Mime {
    mime_guess::from_path(path).first_or_octet_stream()
}

async fn serve_fpm_file(config: &fpm::Config) -> actix_web::HttpResponse {
    let response =
        match tokio::fs::read(config.get_root_for_package(&config.package).join("FPM.ftd")).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("FPM-Error: path: FPM.ftd error: {:?}", e);
                return actix_web::HttpResponse::NotFound().body(e.to_string());
            }
        };
    actix_web::HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(response)
}

async fn static_file(
    req: &actix_web::HttpRequest,
    file_path: camino::Utf8PathBuf,
) -> actix_web::HttpResponse {
    if !file_path.exists() {
        return actix_web::HttpResponse::NotFound().body("".as_bytes());
    }

    match actix_files::NamedFile::open_async(&file_path).await {
        Ok(r) => r.into_response(req),
        Err(e) => {
            eprintln!("FPM-Error: path: {:?}, error: {:?}", file_path, e);
            actix_web::HttpResponse::NotFound().body(e.to_string())
        }
    }
}

async fn serve(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    // TODO: Need to remove unwrap
    let _lock = LOCK.read().await;
    let r = format!("{} {}", req.method().as_str(), req.path());
    let t = fpm::time(r.as_str());
    println!("{r} started");

    let path: camino::Utf8PathBuf = req.match_info().query("path").parse().unwrap();

    let favicon = camino::Utf8PathBuf::new().join("favicon.ico");
    let response = if path.eq(&favicon) {
        static_file(&req, favicon).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("FPM.ftd")) {
        let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
        serve_fpm_file(&config).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("")) {
        let mut config =
            fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
        serve_file(&req, &mut config, &path.join("/")).await
    } else if let Some(cr_number) = fpm::cr::get_cr_path_from_url(path.as_str()) {
        let mut config =
            fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
        serve_cr_file(&req, &mut config, &path, cr_number).await
    } else {
        let mut config =
            fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
        serve_file(&req, &mut config, &path).await
    };

    t.it(response)
}

pub(crate) async fn download_init_package(url: Option<String>) -> std::io::Result<()> {
    let mut package = fpm::Package::new("unknown-package");
    package.download_base_url = url;
    package
        .http_download_by_id(
            "FPM.ftd",
            Some(
                &camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
                    .expect("FPM-Error: Unable to change path"),
            ),
        )
        .await
        .expect("Unable to find FPM.ftd file");
    Ok(())
}

pub async fn clear_cache(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let _lock = LOCK.write().await;
    fpm::apis::cache::clear(&req).await
}

pub async fn handle_wasm(
    // config: fpm::config::Config,
    req: actix_web::HttpRequest,
) -> actix_web::HttpResponse {
    let _lock = LOCK.write().await;
    // let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
    let mut config = wit_bindgen_host_wasmtime_rust::wasmtime::Config::new();
    config.cache_config_load_default().unwrap();
    config.wasm_backtrace_details(
        wit_bindgen_host_wasmtime_rust::wasmtime::WasmBacktraceDetails::Disable,
    );

    // TODO: resolve the path dynamically on the basis of the route
    let wasm_module_path = "/Users/shobhitsharma/repos/playground/hello-wasmer/wit-supabase/target/wasm32-unknown-unknown/release/guest.wasm";
    let engine = wit_bindgen_host_wasmtime_rust::wasmtime::Engine::new(&config).unwrap();
    let module =
        wit_bindgen_host_wasmtime_rust::wasmtime::Module::from_file(&engine, wasm_module_path)
            .unwrap();

    let mut linker: wit_bindgen_host_wasmtime_rust::wasmtime::Linker<
        fpm::wasm_exports::Context<
            fpm::wasm_exports::HostExports,
            fpm_utils::guest::guest::GuestData,
        >,
    > = wit_bindgen_host_wasmtime_rust::wasmtime::Linker::new(&engine);
    let mut store = wit_bindgen_host_wasmtime_rust::wasmtime::Store::new(
        &engine,
        fpm::wasm_exports::Context {
            imports: fpm::wasm_exports::HostExports {},
            exports: fpm_utils::guest::guest::GuestData {},
        },
    );
    fpm_utils::host::host::add_to_linker(&mut linker, |cx| &mut cx.imports);
    fpm_utils::guest::guest::Guest::add_to_linker(&mut linker, |cx| &mut cx.exports);

    let (import, _i) =
        fpm_utils::guest::guest::Guest::instantiate(&mut store, &module, &mut linker, |cx| {
            &mut cx.exports
        })
        .expect("Unable to run");
    let resp = import
        .run(&mut store, "Shobhit")
        .expect("Fn did not execute correctly");

    actix_web::HttpResponse::Ok().body(resp)
    // fpm::apis::cache::clear(&req).await
}

// TODO: Move them to routes folder
async fn sync(
    req: actix_web::web::Json<fpm::apis::sync::SyncRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.write().await;
    fpm::apis::sync(req).await
}

async fn sync2(
    req: actix_web::web::Json<fpm::apis::sync2::SyncRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.write().await;
    fpm::apis::sync2(req).await
}

pub async fn clone() -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.read().await;
    fpm::apis::clone().await
}

pub(crate) async fn view_source(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let _lock = LOCK.read().await;
    fpm::apis::view_source(req).await
}

pub async fn edit(
    req: actix_web::HttpRequest,
    req_data: actix_web::web::Json<fpm::apis::edit::EditRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.write().await;
    fpm::apis::edit(req, req_data).await
}

pub async fn revert(
    req: actix_web::web::Json<fpm::apis::edit::RevertRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.write().await;
    fpm::apis::edit::revert(req).await
}

pub async fn editor_sync() -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.write().await;
    fpm::apis::edit::sync().await
}

pub async fn create_cr(
    req: actix_web::web::Json<fpm::apis::cr::CreateCRRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.write().await;
    fpm::apis::cr::create_cr(req).await
}

pub async fn create_cr_page() -> actix_web::Result<actix_web::HttpResponse> {
    let _lock = LOCK.read().await;
    fpm::apis::cr::create_cr_page().await
}

pub async fn fpm_serve(
    bind_address: &str,
    port: Option<u16>,
    package_download_base_url: Option<String>,
) -> std::io::Result<()> {
    use colored::Colorize;

    if package_download_base_url.is_some() {
        download_init_package(package_download_base_url).await?;
    }

    if cfg!(feature = "controller") {
        // fpm-controller base path and ec2 instance id (hardcoded for now)
        let fpm_controller: String = std::env::var("FPM_CONTROLLER")
            .unwrap_or_else(|_| "https://controller.fifthtry.com".to_string());
        let fpm_instance: String =
            std::env::var("FPM_INSTANCE_ID").expect("FPM_INSTANCE_ID is required");

        println!("Resolving dependency");
        match crate::controller::resolve_dependencies(fpm_instance, fpm_controller).await {
            Ok(_) => println!("Dependencies resolved"),
            Err(e) => panic!("Error resolving dependencies using controller!!: {:?}", e),
        }
    }

    fn get_available_port(port: Option<u16>, bind_address: &str) -> Option<std::net::TcpListener> {
        let available_listener =
            |port: u16, bind_address: &str| std::net::TcpListener::bind((bind_address, port));

        if let Some(port) = port {
            return match available_listener(port, bind_address) {
                Ok(l) => Some(l),
                Err(_) => None,
            };
        }

        for x in 8000..9000 {
            match available_listener(x, bind_address) {
                Ok(l) => return Some(l),
                Err(_) => continue,
            }
        }
        None
    }

    let tcp_listener = match get_available_port(port, bind_address) {
        Some(listener) => listener,
        None => {
            eprintln!(
                "{}",
                port.map(|x| format!(
                    r#"Provided port {} is not available.

You can try without providing port, it will automatically pick unused port."#,
                    x.to_string().red()
                ))
                .unwrap_or_else(|| {
                    "Tried picking port between port 8000 to 9000, none are available :-("
                        .to_string()
                })
            );
            std::process::exit(2);
        }
    };

    let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());

    let app = move || {
        {
            if cfg!(feature = "remote") {
                let json_cfg = actix_web::web::JsonConfig::default()
                    .content_type(|mime| mime == mime_guess::mime::APPLICATION_JSON)
                    .limit(9862416400);

                actix_web::App::new()
                    .app_data(json_cfg)
                    .route("/-/sync/", actix_web::web::post().to(sync))
                    .route("/-/sync2/", actix_web::web::post().to(sync2))
                    .route("/-/clone/", actix_web::web::get().to(clone))
            } else {
                actix_web::App::new()
            }
        }
        .route(
            "/-/view-src/{path:.*}",
            actix_web::web::get().to(view_source),
        )
        .route("/-/edit/", actix_web::web::post().to(edit))
        .route("/-/revert/", actix_web::web::post().to(revert))
        .route("/-/editor-sync/", actix_web::web::get().to(editor_sync))
        .route("/-/create-cr/", actix_web::web::post().to(create_cr))
        .route("/-/create-cr/", actix_web::web::get().to(create_cr_page))
        .route("/-/clear-cache/", actix_web::web::post().to(clear_cache))
        .route("/wasm-hello/", actix_web::web::get().to(handle_wasm))
        .route("/{path:.*}", actix_web::web::get().to(serve))
    };

    println!("### Server Started ###");
    println!(
        "Go to: http://{}:{}",
        bind_address,
        tcp_listener.local_addr()?.port()
    );
    actix_web::HttpServer::new(app)
        .listen(tcp_listener)?
        .run()
        .await
}

// fn authentication(req: &actix_web::HttpRequest) -> bool {
//     false
// }

// cargo install --features controller --path=.
// FPM_CONTROLLER=http://127.0.0.1:8000 FPM_INSTANCE_ID=12345 fpm serve 8001
