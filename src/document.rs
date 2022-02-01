#[derive(Debug, Clone)]
pub enum File {
    Ftd(Document),
    Static(Static),
    Markdown(Document),
}

impl File {
    pub fn get_id(&self) -> String {
        match self {
            Self::Ftd(a) => a.id.clone(),
            Self::Static(a) => a.id.clone(),
            Self::Markdown(a) => a.id.clone(),
        }
    }
    pub fn get_base_path(&self) -> String {
        match self {
            Self::Ftd(a) => a.parent_path.to_string(),
            Self::Static(a) => a.base_path.to_string(),
            Self::Markdown(a) => a.parent_path.to_string(),
        }
    }
    pub fn get_full_path(&self) -> camino::Utf8PathBuf {
        let (id, base_path) = match self {
            Self::Ftd(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Static(a) => (a.id.to_string(), a.base_path.to_string()),
            Self::Markdown(a) => (a.id.to_string(), a.parent_path.to_string()),
        };
        camino::Utf8PathBuf::from(base_path).join(id)
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub parent_path: String,
}

impl Document {
    pub fn id_to_path(&self) -> String {
        self.id
            .replace(".ftd", std::path::MAIN_SEPARATOR.to_string().as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Static {
    pub id: String,
    pub base_path: camino::Utf8PathBuf,
}

pub(crate) async fn get_documents(config: &fpm::Config) -> fpm::Result<Vec<fpm::File>> {
    let p = match &config.package.root_directory {
        Some(p) => p.replace("/", std::path::MAIN_SEPARATOR.to_string().as_str()),
        None => "".to_string(),
    };
    let root_dir = config.root.clone().join(p);

    let mut ignore_paths = ignore::WalkBuilder::new(&root_dir);
    ignore_paths.overrides(package_ignores()?); // unwrap ok because this we know can never fail
    ignore_paths.standard_filters(true);
    ignore_paths.overrides(config.ignored.clone());
    let all_files = ignore_paths
        .build()
        .into_iter()
        .flatten()
        .map(|x| camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap()) //todo: improve error message
        .collect::<Vec<camino::Utf8PathBuf>>();
    let fpm_ftd_path = config.root.clone().join("FPM.ftd");
    let mut documents = if all_files.contains(&fpm_ftd_path) {
        fpm::paths_to_files(all_files, &root_dir).await?
    } else {
        let mut documents = fpm::paths_to_files(all_files, &root_dir).await?;
        let global_docs = fpm::paths_to_files(vec![fpm_ftd_path], &config.root).await?;
        documents.extend(global_docs);
        documents
    };

    documents.sort_by_key(|v| v.get_id());

    Ok(documents)
}

pub(crate) async fn paths_to_files(
    files: Vec<camino::Utf8PathBuf>,
    base_path: &camino::Utf8Path,
) -> fpm::Result<Vec<fpm::File>> {
    Ok(futures::future::join_all(
        files
            .into_iter()
            .map(|x| {
                let base = base_path.to_path_buf();
                tokio::spawn(async move { fpm::get_file(&x, &base).await })
            })
            .collect::<Vec<tokio::task::JoinHandle<fpm::Result<fpm::File>>>>(),
    )
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect::<Vec<fpm::File>>())
}

pub fn package_ignores() -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new("./");
    overrides.add("!.history")?;
    overrides.add("!.packages")?;
    overrides.add("!.tracks")?;
    overrides.add("!FPM")?;
    overrides.add("!rust-toolchain")?;
    overrides.add("!.build")?;
    overrides.build()
}

pub(crate) async fn get_file(
    doc_path: &camino::Utf8Path,
    base_path: &camino::Utf8Path,
) -> fpm::Result<File> {
    if doc_path.is_dir() {
        return Err(fpm::Error::UsageError {
            message: format!("{} should be a file", doc_path.as_str()),
        });
    }

    let id = match std::fs::canonicalize(doc_path)?
        .to_str()
        .unwrap()
        .rsplit_once(
            if base_path.as_str().ends_with(std::path::MAIN_SEPARATOR) {
                base_path.as_str().to_string()
            } else {
                format!("{}{}", base_path, std::path::MAIN_SEPARATOR)
            }
            .as_str(),
        ) {
        Some((_, id)) => id.to_string(),
        None => {
            return Err(fpm::Error::UsageError {
                message: format!("{:?} should be a file", doc_path),
            });
        }
    };

    Ok(match id.rsplit_once(".") {
        Some((_, "ftd")) => File::Ftd(Document {
            id: id.to_string(),
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
        }),
        Some((_, "md")) => File::Markdown(Document {
            id: id.to_string(),
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
        }),
        _ => File::Static(Static {
            id: id.to_string(),
            base_path: base_path.to_path_buf(),
        }),
    })
}
