pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(ref sitemap) = config.sitemap {
        let mut doc_id = config
            .current_document
            .clone()
            .map(|v| fpm::utils::id_to_path(v.as_str()))
            .unwrap_or_else(|| {
                doc.name
                    .to_string()
                    .replace(config.package.name.as_str(), "")
            })
            .trim()
            .replace(std::path::MAIN_SEPARATOR, "/");

        if !config.package.versioned.eq("false") {
            let versions = futures::executor::block_on(config.get_versions(&config.package))
                .map_err(|e| ftd::p1::Error::ParseError {
                    message: format!("Cant find versions: {:?}", e),
                    doc_id: doc.name.to_string(),
                    line_number: section.line_number,
                })?;
            for version in versions.keys() {
                let base = if let Some(ref base) = version.base {
                    if base.is_empty() {
                        base.to_string()
                    } else {
                        format!("{}/", base)
                    }
                } else {
                    "".to_string()
                };
                if let Some(id) = doc_id
                    .trim_matches('/')
                    .strip_prefix(format!("{}{}", base, version.original).as_str())
                {
                    doc_id = format!("{}{}", base, id.trim_matches('/'));
                    break;
                }
            }
        }

        if let Some(sitemap) = sitemap.get_sitemap_by_id(doc_id.as_str()) {
            return doc.from_json(&sitemap, section);
        }
    }
    doc.from_json(&fpm::sitemap::SiteMapCompat::default(), section)
}
