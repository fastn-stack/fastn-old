async fn handle_ftd(config: &mut fpm::Config, path: std::path::PathBuf) -> actix_web::HttpResponse {
    use itertools::Itertools;
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
        config.package.get_assets_doc(config, "/").await.unwrap(),
    );

    for dep in &dependencies {
        asset_documents.insert(
            dep.package.name.clone(),
            dep.package.get_assets_doc(config, "/").await.unwrap(),
        );
    }

    let new_path = match path.to_str() {
        Some(s) => s.replace("-/", ""),
        None => {
            println!("handle_ftd: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    let dep_package = find_dep_package(config, &dependencies, &new_path);

    let f = match config.get_file_by_id(&new_path, dep_package).await {
        Ok(f) => f,
        Err(e) => {
            println!("path: {}, Error: {:?}", new_path, e);
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
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
                &asset_documents,
                false,
            )
            .await
            {
                Ok(r) => actix_web::HttpResponse::Ok().body(r),
                Err(_e) => actix_web::HttpResponse::InternalServerError().body("TODO".as_bytes()),
            };
        }
        _ => actix_web::HttpResponse::InternalServerError().body("".as_bytes()),
    };

    fn find_dep_package<'a>(
        config: &'a fpm::Config,
        dep: &'a [fpm::Dependency],
        file_path: &'a str,
    ) -> &'a fpm::Package {
        dep.iter()
            .find(|d| file_path.starts_with(&d.package.name))
            .map(|x| &x.package)
            .unwrap_or(&config.package)
    }
}

async fn handle_dash(
    req: &actix_web::HttpRequest,
    config: &fpm::Config,
    path: std::path::PathBuf,
) -> actix_web::HttpResponse {
    let new_path = match path.to_str() {
        Some(s) => s.replace("-/", ""),
        None => {
            println!("handle_dash: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    let file_path = if new_path.starts_with(&config.package.name) {
        std::path::PathBuf::new().join(
            new_path
                .strip_prefix(&(config.package.name.to_string() + "/"))
                .unwrap(),
        )
    } else {
        std::path::PathBuf::new().join(".packages").join(new_path)
    };

    server_static_file(req, file_path).await
}

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
    let mut config = fpm::Config::read(None).await.unwrap();
    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();

    let favicon = std::path::PathBuf::new().join("favicon.ico");
    if path.starts_with("-/") {
        handle_dash(&req, &config, path).await
    } else if path.eq(&favicon) {
        server_static_file(&req, favicon).await
    } else if path.eq(&std::path::PathBuf::new().join("")) {
        handle_ftd(&mut config, path.join("index")).await
    } else {
        handle_ftd(&mut config, path).await
    }
}

#[actix_web::main]
pub async fn serve(port: &str) -> std::io::Result<()> {
    // For debugging (will be refactored later)
    let use_controller = false;

    if use_controller {
        // fpm-controller base path and ec2 instance id (hardcoded for now)
        let fpm_controller: String = "https:///controller.fifthtry.com".to_string();
        let fpm_instance: String = "<some-ec2-instance-id>".to_string();

        match controller::resolve_dependencies(fpm_instance, fpm_controller).await {
            Ok(_) => println!("Dependencies resolved"),
            Err(_) => panic!("Error resolving dependencies using controller!!"),
        }
    }

    println!("### Server Started ###");
    println!("Go to: http://127.0.0.1:{}", port);
    actix_web::HttpServer::new(|| {
        actix_web::App::new().route("/{path:.*}", actix_web::web::get().to(serve_static))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}

// FPM Controller Support
// FPM cli supports communication with fpm controller. This is an optional feature, and is only available when controller feature is enabled, which is not enabled by default.
// Controller Communication
// When controller feature is enabled, fpm serve will first communicate with the FPM controller service’s /get-package/ API.

// FPM Controller Service Endpoint
// The FPM Controller Service’s endpoint is computed by using environment variable FPM_CONTROLLER, which will look something like this: https://controller.fifthtry.com, with the API path.
// FPM Controller Service has more than one APIs: /get-package/ and /fpm-ready/.

// get-package:
// Through an environment variable FPM_INSTANCE_ID, the fpm serve will learn it’s instance id, and it will pass the instance id to the get-package API.
// The API returns the URL of the package to be downloaded, git repository URL and the package name.
// FPM will clone the git repository in the current directory. The current directory will contain FPM.ftd and other files of the package.
// FPM will then calls fpm install on it.

// fpm-ready:
// Once dependencies are ready fpm calls /fpm-ready/ API on the controller. We will pass the FPM_INSTANCE_ID and the git commit hash as input to the API
// The API will return with success, and once it is done fpm will start receiving HTTP traffic from the controller service.

mod controller {
    pub async fn resolve_dependencies(
        fpm_instance: String,
        fpm_controller: String,
    ) -> fpm::Result<()> {
        // First call get_package API to get package details and resolve dependencies

        // response from get-package API
        let package_response = get_package(fpm_instance.as_str(), fpm_controller.as_str()).await?;
        let gp_status = match package_response["success"].as_bool() {
            Some(res) => res,
            None => panic!("success parameter doesn't exist in Json or isn't valid boolean type"),
        };

        match gp_status {
            true => {
                // package name and git repo url
                let package_name = match package_response["result"]["package"].as_str() {
                    Some(valid_name) => valid_name,
                    None => panic!("received invalid package name from get_package API"),
                };
                if let Some(git_url) = package_response["result"]["git"].as_str() {
                    // Clone the git package into the current directory
                    // Need to execute shell commands from rust
                    // git_url https format: https://github.com/<user>/<repo>.git
                    let out = std::process::Command::new("git")
                        .arg("clone")
                        .arg(git_url)
                        .output()
                        .expect("unable to execute git clone command");

                    if out.status.success() {
                        // By this time the cloned repo should be available in the current directory
                        println!("Git cloning successful for the package {}", package_name);
                        // Resolve dependencies by reading the FPM.ftd using config.read()
                        // Assuming package_name and repo name are identical
                        let _config = fpm::Config::read(Some(package_name.to_string())).await?;
                    }
                } else {
                    panic!("Invalid git url for the package {}", package_name);
                }
            },
            false => panic!("get-package api success status returned false!!")
        }

        // Once the dependencies are resolved for the package
        // then call fpm_ready API to ensure that the controller service is now ready

        // response from fpm_ready API
        let ready_response = fpm_ready(fpm_instance.as_str(), fpm_controller.as_str()).await?;
        let fr_status = match ready_response["success"].as_bool() {
            Some(res) => res,
            None => panic!("success parameter doesn't exist in Json or isn't valid boolean type"),
        };

        match fr_status {
            true => println!("FPM controller ready!!"),
            false => panic!("FPM controller isn't ready!!"),
        }

        Ok(())
    }

    async fn make_get_request(url: url::Url) -> fpm::Result<serde_json::Value> {
        use fpm::library::http;
        let response = match http::_get(url).await {
            Ok(some_response) => some_response,
            Err(e) => {
                panic!("failed to fetch data, error: {}", e.to_string())
            }
        };

        match serde_json::from_str(response.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => panic!(
                "failed parsing json from response text, error: {}",
                e.to_string()
            ),
        }
    }

    async fn get_package(
        fpm_instance: &str,
        fpm_controller: &str,
    ) -> fpm::Result<serde_json::Value> {
        // get-package API
        // input: fpm_instance
        // output: package_name and git repo URL
        // format: {
        //     "success": true,
        //     "result": {
        //         "package": "<package name>"
        //         "git": "<git url>"
        //     }
        // }

        let query_string = format!("instance={}", fpm_instance);
        let controller_api = format!("{}/get-package?{}", fpm_controller, query_string);

        let url = match url::Url::parse(controller_api.as_str()) {
            Ok(safe_url) => safe_url,
            Err(e) => panic!("Invalid get-package API endpoint, Parse error: {}",e.to_string()),
        };

        make_get_request(url).await
    }

    async fn fpm_ready(fpm_instance: &str, fpm_controller: &str) -> fpm::Result<serde_json::Value> {
        // fpm-ready API
        // input: fpm_instance, *(git commit hash)
        // output: success: true/false
        // format: lang: json
        // {
        //     "success": true
        // }

        // Git commit hash needs to be computed before making a call to the fpm_ready API
        let git_commit = "<dummy-git-commit-hash-xxx123>";

        let query_string = format!("instance={}&git-commit={}", fpm_instance, git_commit);
        let controller_api = format!("{}/fpm-ready?{}", fpm_controller, query_string);

        let url = match url::Url::parse(controller_api.as_str()) {
            Ok(safe_url) => safe_url,
            Err(e) => panic!("Invalid fpm_ready API endpoint, Parse error: {}",e.to_string()),
        };

        // This request should be put request for fpm_ready API to update the instance status to ready
        // Using http::_get() function to make request to this API for now
        make_get_request(url).await
    }
}
