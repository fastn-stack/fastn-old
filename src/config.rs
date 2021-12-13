#[derive(Debug)]
pub struct Config {
    pub package: fpm::Package,
    pub root: camino::Utf8PathBuf,
    pub original_directory: camino::Utf8PathBuf,
    pub fonts: Vec<fpm::Font>,
    pub dependencies: Vec<fpm::Dependency>,
    pub ignored: ignore::overrides::Override,
}

impl Config {
    pub fn build_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".build")
    }

    pub fn latest_ftd(&self) -> camino::Utf8PathBuf {
        self.root.join(".history/.latest.ftd")
    }

    pub fn get_font_style(&self) -> String {
        let generated_style = self
            .fonts
            .iter()
            .fold("".to_string(), |c, f| format!("{}\n{}", c, f.to_html()));
        return match generated_style.trim().is_empty() {
            false => format!("<style>{}</style>", generated_style),
            _ => format!(""),
        };
    }

    pub async fn read() -> fpm::Result<Config> {
        let original_directory: camino::Utf8PathBuf =
            std::env::current_dir()?.canonicalize()?.try_into()?;
        let root = find_package_root(&original_directory)?;

        std::env::set_current_dir(&root);

        let ftd_doc = {
            let doc = tokio::fs::read_to_string(root.join("FPM.ftd")).await?;
            let lib = fpm::Library::default();
            ftd::p2::Document::from("FPM", doc.as_str(), &lib)?
        };

        let package = {
            let package: fpm::Package = ftd_doc.get("fpm#package")?;
            if root.file_name() != Some(package.name.as_str()) {
                return Err(fpm::Error::ConfigurationError {
                    message: "package name and folder name must match".to_string(),
                });
            }
            package
        };

        let deps: Vec<fpm::Dependency> = ftd_doc.get("fpm#dependency")?;
        let fonts: Vec<fpm::Font> = ftd_doc.get("fpm#font")?;

        let ignored = {
            let mut overrides = ignore::overrides::OverrideBuilder::new("./");
            for ig in ftd_doc.get::<Vec<String>>("fpm#ignore")? {
                if let Err(e) = overrides.add(format!("!{}", ig.as_str()).as_str()) {
                    return Err(fpm::Error::ConfigurationError {
                        message: format!("failed parse fpm.ignore: {} => {:?}", ig, e),
                    });
                }
            }

            match overrides.build() {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::ConfigurationError {
                        message: format!("failed parse fpm.ignore: {:?}", e),
                    });
                }
            }
        };

        fpm::dependency::ensure(root.clone(), deps.clone()).await?;

        Ok(Config {
            package,
            root,
            original_directory,
            fonts,
            dependencies: deps,
            ignored,
        })
    }
}

fn find_package_root(dir: &camino::Utf8Path) -> fpm::Result<camino::Utf8PathBuf> {
    if dir.join("FPM.ftd").exists() {
        Ok(dir.into())
    } else if let Some(p) = dir.parent() {
        find_package_root(p)
    } else {
        Err(fpm::Error::ConfigurationError {
            message: "FPM.ftd not found in any parent directory".to_string(),
        })
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub about: Option<String>,
    pub domain: Option<String>,
}
