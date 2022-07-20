#[derive(serde::Serialize)]
pub struct TocItemUI {
    pub id: String,
    pub title: Option<String>,
    pub extra_data: Option<String>,
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub children: Vec<TocItemUI>,
    pub skip: bool,
}

#[derive(serde::Serialize)]
struct SubSectionCompat {
    pub id: Option<String>,
    pub title: Option<String>,
    pub visible: bool,
    pub extra_data: Option<String>, // TODO: Need to convert it into map
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub toc: Vec<TocItemUI>,
    pub skip: bool,
}

#[derive(serde::Serialize)]
struct SectionCompat {
    id: String,
    title: Option<String>,
    extra_data: Option<String>, // TODO: Need to convert it into map
    is_active: bool,
    nav_title: Option<String>,
    subsections: Vec<SubSectionCompat>,
}

#[derive(serde::Serialize)]
struct SitemapUI {
    sections: Vec<SectionUI>,
}

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    //TODO: Need to convert sitemap to sitemap compat
    if let Some(ref sitemap) = config.sitemap {
        return doc.from_json(&sitemap, section);
    }
    doc.from_json(&fpm::sitemap::SiteMapCompat::default(), section)
}
