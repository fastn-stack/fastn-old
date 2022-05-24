use itertools::Itertools;

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
    document_id: &str,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    let versions =
        futures::executor::block_on(config.get_versions(&config.package)).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: format!("Cant find versions: {:?}", e),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }
        })?;

    let version =
        fpm::Version::parse(document_id, config.package.versioned.as_str()).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: format!("{:?}", e),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }
        })?;

    let url = config
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

    let doc_id = {
        let mut doc_id = document_id.trim_start_matches('/').to_string();
        if let Some(id) = doc_id.strip_prefix(base_url.trim().trim_matches('/')) {
            doc_id = id.trim_start_matches('/').to_string();
        }
        doc_id
    };
    let mut found = false;
    if let Some(doc) = versions.get(&fpm::Version::base_(version.base.to_owned())) {
        if doc.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
            found = true;
        }
    }

    let mut version_toc = vec![];
    for key in versions
        .iter()
        .filter(|(v, _)| v.base.eq(&version.base))
        .map(|(v, _)| v)
        .sorted()
    {
        if key.eq(&fpm::Version::base()) {
            continue;
        }
        let doc = versions[key].to_owned();
        if !found {
            if !doc.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
                continue;
            }
            found = true;
        }

        let url = {
            let mut url = url.trim_start_matches('/').to_string();
            if let Some(id) = url.strip_prefix(base_url.trim().trim_matches('/')) {
                url = id.trim_start_matches('/').to_string();
            }
            url
        };

        version_toc.push(fpm::library::toc::TocItem {
            id: None,
            title: Some(key.original.to_string()),
            url: Some(format!(
                "{}{}/{}",
                if let Some(ref base) = key.base {
                    format!("/{}/", base)
                } else {
                    "/".to_string()
                },
                key.original,
                url
            )),
            number: vec![],
            is_heading: version.eq(key),
            is_disabled: false,
            img_src: None,
            font_icon: None,
            children: vec![],
        });
    }

    let toc_items = version_toc
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<fpm::library::toc::TocItemCompat>>();

    doc.from_json(&toc_items, section)
}
