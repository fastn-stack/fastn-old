pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    //TODO: Need to convert sitemap to sitemap compat
    // if let Some(ref sitemap) = config.sitemap {
    //     return doc.from_json(&sitemap, section);
    // }
    doc.from_json(&fpm::sitemap::SiteMapCompat::default(), section)
}
