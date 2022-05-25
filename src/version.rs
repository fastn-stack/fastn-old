use itertools::Itertools;
use std::cmp::Ordering;
use std::io::Write;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Version {
    pub major: u64,
    pub minor: Option<u64>,
    pub original: String,
    pub base: Option<String>,
}

impl Version {
    pub(crate) fn base() -> fpm::Version {
        fpm::Version::base_(None)
    }

    pub(crate) fn base_(base: Option<String>) -> fpm::Version {
        fpm::Version {
            major: 0,
            minor: None,
            original: "BASE_VERSION".to_string(),
            base,
        }
    }

    pub(crate) fn parse(s: &str, versioned: &str) -> fpm::Result<fpm::Version> {
        let get_all_version_base = if versioned.eq("true") {
            vec![]
        } else {
            versioned
                .split(',')
                .filter(|v| !v.is_empty())
                .map(|v| v.trim())
                .collect()
        };

        let mut base = None;
        let mut s = s.to_string();
        for version_base in get_all_version_base.iter() {
            if let Some(id) = s.trim_matches('/').strip_prefix(version_base) {
                s = id.trim_matches('/').to_string();
                base = Some(version_base.to_string());
                break;
            }
        }

        if !get_all_version_base.is_empty() && base.is_none() && !s.eq("FPM.ftd") {
            return Err(fpm::Error::UsageError {
                message: format!(
                    "{} is not part of any versioned directory: Available versioned directories: {:?}",
                    s, get_all_version_base
                ),
            });
        }

        if let Some((v, _)) = s.split_once('/') {
            s = v.to_string();
        } else {
            return Ok(fpm::Version::base_(base));
        }

        let v = s.strip_prefix(&['v', 'V']).unwrap_or_else(|| s.as_str());
        let mut minor = None;
        let major = if let Some((major, minor_)) = v.split_once('.') {
            if minor_.contains('.') {
                return Err(fpm::Error::UsageError {
                    message: format!("Cannot have more than one dots `.`, found: `{}`", s),
                });
            }
            let minor_ = minor_.parse::<u64>().map_err(|e| fpm::Error::UsageError {
                message: format!("Invalid minor for `{}`: `{:?}`", s, e),
            })?;
            minor = Some(minor_);
            major.parse::<u64>().map_err(|e| fpm::Error::UsageError {
                message: format!("Invalid major for `{}`: `{:?}`", s, e),
            })?
        } else {
            v.parse::<u64>().map_err(|e| fpm::Error::UsageError {
                message: format!("Invalid major for `{}`: `{:?}`", s, e),
            })?
        };
        Ok(fpm::Version {
            major,
            minor,
            original: s.to_string(),
            base,
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, rhs))
    }
}

impl Ord for Version {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        if self.major.eq(&rhs.major) {
            let lhs_minor = self.minor.unwrap_or(0);
            let rhs_minor = rhs.minor.unwrap_or(0);
            return lhs_minor.cmp(&rhs_minor);
        }
        self.major.cmp(&rhs.major)
    }
}

pub(crate) async fn build_version(
    config: &mut fpm::Config,
    _file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let mut all_canonical = config
        .sitemap
        .as_ref()
        .map(|v| v.get_all_canonical_ids())
        .unwrap_or(Default::default());

    let base_versioned_documents = config.get_based_versions(&config.package).await?;

    for (base, versioned_documents) in base_versioned_documents.iter() {
        let base = match base.as_ref() {
            "BASE" => "".to_string(),
            v => format!("{}/", v),
        };

        let mut documents = std::collections::BTreeMap::new();
        for key in versioned_documents.keys().sorted() {
            let doc = {
                let mut doc = versioned_documents[key].to_owned();
                let mut variants = vec![];
                for doc in doc.iter() {
                    let id = doc.get_id();
                    if id.eq("FPM.ftd") {
                        continue;
                    }
                    let path = fpm::utils::id_to_path(id.as_str())
                        .trim_start_matches('/')
                        .to_string();
                    let find_id = format!("{}{}", base, path);
                    if let Some(variant) = all_canonical.remove(find_id.as_str()) {
                        let variant = if variant.ends_with('/') {
                            variant
                        } else {
                            format!("{}/", variant)
                        };
                        let mut doc = doc.clone();
                        let extension = if matches!(doc, fpm::File::Markdown(_)) {
                            "index.md".to_string()
                        } else {
                            "index.ftd".to_string()
                        };

                        doc.set_id(format!("{}-/{}{}", path, variant, extension).as_str());
                        variants.push(doc);
                    }
                }
                doc.extend(variants);
                doc
            };
            documents.extend(
                doc.iter()
                    .map(|v| (v.get_id(), (key.original.to_string(), v.to_owned()))),
            );
            if key.original.eq(&fpm::Version::base().original) {
                continue;
            }
            for (version, doc) in documents.values() {
                let mut doc = doc.clone();
                let id = doc.get_id();
                if id.eq("FPM.ftd") {
                    continue;
                }

                let new_id = format!("{}{}/{}", base, key.original, id);
                if !key.original.eq(version) && !fpm::Version::base().original.eq(version) {
                    if let fpm::File::Ftd(_) = doc {
                        let original_id = format!("{}{}/{}", base, version, id);
                        let original_file_rel_path = if original_id.contains("index.ftd") {
                            original_id.replace("index.ftd", "index.html")
                        } else {
                            original_id.replace(
                                ".ftd",
                                format!("{}index.html", std::path::MAIN_SEPARATOR).as_str(),
                            )
                        };
                        let original_file_path =
                            config.root.join(".build").join(original_file_rel_path);
                        let file_rel_path = if new_id.contains("index.ftd") {
                            new_id.replace("index.ftd", "index.html")
                        } else {
                            new_id.replace(
                                ".ftd",
                                format!("{}index.html", std::path::MAIN_SEPARATOR).as_str(),
                            )
                        };
                        let new_file_path = config.root.join(".build").join(file_rel_path);
                        let original_content = std::fs::read_to_string(&original_file_path)?;
                        std::fs::create_dir_all(&new_file_path.as_str().replace("index.html", ""))?;
                        let mut f = std::fs::File::create(&new_file_path)?;
                        let from_pattern = format!("<base href=\"{}{}/\">", base_url, version);
                        let to_pattern = format!("<base href=\"{}{}/\">", base_url, key.original);
                        f.write_all(
                            original_content
                                .replace(from_pattern.as_str(), to_pattern.as_str())
                                .as_bytes(),
                        )?;
                        continue;
                    }
                }
                doc.set_id(new_id.as_str());
                config.current_document = Some(new_id);
                fpm::process_file(
                    config,
                    &config.package,
                    &doc,
                    None,
                    None,
                    Default::default(),
                    format!("{}{}{}/", base_url, base, key.original).as_str(),
                    skip_failed,
                    asset_documents,
                    Some(id),
                    false,
                )
                .await?;
            }
        }
        for (_, doc) in documents.values_mut() {
            let id = format!("{}{}", base, doc.get_id());
            doc.set_id(id.as_str());
            config.current_document = Some(id);
            fpm::process_file(
                config,
                &config.package,
                doc,
                None,
                None,
                Default::default(),
                format!("{}{}", base_url, base).as_str(),
                skip_failed,
                asset_documents,
                None,
                false,
            )
            .await?;
        }
    }
    Ok(())
}
