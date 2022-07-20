use itertools::Itertools;

#[derive(Default, Debug, serde::Serialize)]
pub struct TocItemCompat {
    pub id: String,
    pub title: Option<String>,
    pub extra_data: Option<String>,
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub children: Vec<TocItemCompat>,
    pub skip: bool,
}

#[derive(Default, Debug, serde::Serialize)]
struct SubSectionCompat {
    pub id: Option<String>,
    pub title: Option<String>,
    pub visible: bool,
    pub extra_data: Option<String>,
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub toc: Vec<TocItemCompat>,
    pub skip: bool,
}

#[derive(Default, Debug, serde::Serialize)]
struct SectionCompat {
    id: String,
    title: Option<String>,
    extra_data: Option<String>, // TODO: Need to convert it into map
    is_active: bool,
    nav_title: Option<String>,
    subsections: Vec<SubSectionCompat>,
}

#[derive(Default, Debug, serde::Serialize)]
struct SiteMapCompat {
    sections: Vec<SectionCompat>,
}

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(ref sitemap) = config.sitemap {
        return doc.from_json(&to_sitemap_compat(sitemap), section);
    }
    doc.from_json(&SiteMapCompat::default(), section)
}

fn to_sitemap_compat(sitemap: &fpm::sitemap::Sitemap) -> SiteMapCompat {
    fn to_toc_compat(toc_item: &fpm::sitemap::TocItem) -> TocItemCompat {
        let toc_compat = TocItemCompat {
            id: toc_item.id.clone(),
            title: toc_item.title.clone(),
            extra_data: None,
            is_active: toc_item.is_active,
            nav_title: toc_item.nav_title.clone(),
            children: toc_item.children.iter().map(to_toc_compat).collect_vec(),
            skip: toc_item.skip,
        };
        toc_compat
    }

    fn to_subsection_compat(subsection: &fpm::sitemap::Subsection) -> SubSectionCompat {
        SubSectionCompat {
            id: subsection.id.clone(),
            title: subsection.title.clone(),
            visible: subsection.visible,
            extra_data: None,
            is_active: subsection.is_active,
            nav_title: subsection.nav_title.clone(),
            toc: subsection.toc.iter().map(to_toc_compat).collect_vec(),
            skip: subsection.skip,
        }
    }

    fn to_section_compat(section: &fpm::sitemap::Section) -> SectionCompat {
        SectionCompat {
            id: section.id.to_string(),
            title: section.title.clone(),
            extra_data: None,
            is_active: section.is_active,
            nav_title: section.nav_title.clone(),
            subsections: section
                .subsections
                .iter()
                .map(to_subsection_compat)
                .collect_vec(),
        }
    }

    SiteMapCompat {
        sections: sitemap.sections.iter().map(to_section_compat).collect_vec(),
    }
}
