use tokio::io::AsyncWriteExt;

impl fpm::Package {
    pub(crate) async fn fs_fetch_by_file_name(
        &self,
        name: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<Vec<u8>> {
        let package_root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };
        Ok(tokio::fs::read(package_root.join(name)).await?)
    }

    pub(crate) async fn fs_fetch_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<(String, Vec<u8>)> {
        for name in file_id_to_names(id) {
            if let Ok(data) = self
                .fs_fetch_by_file_name(name.as_str(), package_root)
                .await
            {
                return Ok((name, data));
            }
        }
        Err(fpm::Error::PackageError {
            message: format!(
                "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                id, &self.name
            ),
        })
    }

    async fn http_fetch_by_file_name(&self, name: &str) -> fpm::Result<Vec<u8>> {
        let base = self.base.as_ref().ok_or_else(|| fpm::Error::PackageError {
            message: format!(
                "package base not found. Package: {}, File: {}",
                &self.name, name
            ),
        })?;
        fpm::utils::construct_url_and_get(
            format!("{}/{}", base.trim_end_matches('/'), name).as_str(),
        )
        .await
    }

    async fn http_fetch_by_id(&self, id: &str) -> fpm::Result<(String, Vec<u8>)> {
        for name in file_id_to_names(id) {
            if let Ok(data) = self.http_fetch_by_file_name(name.as_str()).await {
                return Ok((name, data));
            }
        }

        Err(fpm::Error::PackageError {
            message: format!(
                "http_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                id, &self.name
            ),
        })
    }

    pub(crate) async fn http_download_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<(String, Vec<u8>)> {
        let package_root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };

        let (file_path, data) = self.http_fetch_by_id(id).await?;

        let (file_root, file_name) =
            if let Some((file_root, file_name)) = file_path.rsplit_once('/') {
                (file_root.to_string(), file_name.to_string())
            } else {
                ("".to_string(), file_path.to_string())
            };

        tokio::fs::create_dir_all(package_root.join(&file_root)).await?;

        tokio::fs::File::create(package_root.join(file_root).join(file_name))
            .await?
            .write_all(data.as_slice())
            .await?;

        Ok((file_path, data))
    }

    pub(crate) async fn http_download_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<Vec<u8>> {
        let package_root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };

        let data = self.http_fetch_by_file_name(file_path).await?;

        let (file_root, file_name) =
            if let Some((file_root, file_name)) = file_path.rsplit_once('/') {
                (file_root.to_string(), file_name.to_string())
            } else {
                ("".to_string(), file_path.to_string())
            };

        tokio::fs::create_dir_all(package_root.join(&file_root)).await?;

        tokio::fs::File::create(package_root.join(file_root).join(file_name))
            .await?
            .write_all(data.as_slice())
            .await?;

        Ok(data)
    }

    pub(crate) async fn resolve_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<Vec<u8>> {
        if let Ok(response) = self.fs_fetch_by_file_name(file_path, package_root).await {
            return Ok(response);
        }
        if let Ok(response) = self
            .http_download_by_file_name(file_path, package_root)
            .await
        {
            return Ok(response);
        }

        let file_path = match file_path.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && remaining.ends_with("-dark") =>
            {
                format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
            }
            _ => {
                return Err(fpm::Error::PackageError {
                    message: format!(
                        "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                        file_path, &self.name
                    ),
                })
            }
        };

        dbg!("resolve_by_file_name::", &file_path);

        if let Ok(response) = self
            .fs_fetch_by_file_name(file_path.as_str(), package_root)
            .await
        {
            return Ok(response);
        }

        self.http_download_by_file_name(file_path.as_str(), package_root)
            .await
    }

    pub(crate) async fn resolve_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<(String, Vec<u8>)> {
        if let Ok(response) = self.fs_fetch_by_id(id, package_root).await {
            return Ok(response);
        }

        if let Ok(response) = self.http_download_by_id(id, package_root).await {
            return Ok(response);
        }

        let new_id = match id.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && remaining.ends_with("-dark") =>
            {
                format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
            }
            _ => {
                return Err(fpm::Error::PackageError {
                    message: format!(
                        "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                        id, &self.name
                    ),
                })
            }
        };

        dbg!("resolve_by_id::", &new_id);
        if let Ok(response) = self.fs_fetch_by_id(new_id.as_str(), package_root).await {
            return Ok(response);
        }

        self.http_download_by_id(new_id.as_str(), package_root)
            .await
    }
}

fn file_id_to_names(id: &str) -> Vec<String> {
    let id = id.replace("/index.html", "/").replace("index.html", "/");
    if id.eq("/") {
        return vec![
            "index.ftd".to_string(),
            "README.md".to_string(),
            "index.md".to_string(),
        ];
    }
    let mut ids = vec![];
    if !id.ends_with('/') {
        ids.push(id.trim_matches('/').to_string());
    }
    let id = id.trim_matches('/').to_string();
    ids.extend([
        format!("{}.ftd", id),
        format!("{}/index.ftd", id),
        format!("{}.md", id),
        format!("{}/README.md", id),
        format!("{}/index.md", id),
    ]);
    ids
}

pub(crate) async fn read_ftd(
    config: &mut fpm::Config,
    main: &fpm::Document,
    base_url: &str,
) -> fpm::Result<Vec<u8>> {
    let current_package = config
        .all_packages
        .get(main.package_name.as_str())
        .unwrap_or(&config.package);

    let mut lib = fpm::Library2 {
        config: config.clone(),
        markdown: None,
        document_id: main.id.clone(),
        translated_data: Default::default(),
        base_url: base_url.to_string(),
        packages_under_process: vec![current_package.name.to_string()],
    };

    let main_ftd_doc = match fpm::doc::parse2(
        main.id_with_package().as_str(),
        current_package
            .get_prefixed_body(main.content.as_str(), &main.id, true)
            .as_str(),
        &mut lib,
        base_url,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(fpm::Error::PackageError {
                message: format!("failed to parse {:?}", &e),
            });
        }
    };

    let doc_title = match &main_ftd_doc.title() {
        Some(x) => x.original.clone(),
        _ => main.id.as_str().to_string(),
    };
    let ftd_doc = main_ftd_doc.to_rt("main", &main.id);

    let file_content = fpm::utils::replace_markers(
        fpm::ftd_html(),
        config,
        main.id_to_path().as_str(),
        doc_title.as_str(),
        base_url,
        &ftd_doc,
    );

    Ok(file_content.into())
}
