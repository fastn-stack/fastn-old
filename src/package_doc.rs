use tokio::io::AsyncWriteExt;

impl fpm::Package {
    async fn fs_fetch_by_file_name(
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
        fpm::utils::construct_url_and_get_bytes(
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
}

fn file_id_to_names(id: &str) -> Vec<String> {
    if id.eq("/") {
        return vec![
            "index.ftd".to_string(),
            "README.md".to_string(),
            "index.md".to_string(),
        ];
    }
    let id = id.trim_matches('/').to_string();
    vec![
        format!("{}.ftd", id),
        format!("{}/index.ftd", id),
        format!("{}.md", id),
        format!("{}/README.md", id),
        format!("{}/index.md", id),
    ]
}
