#[derive(serde::Deserialize, Debug, Default)]
pub struct CRAbout {
    pub title: String, // relative file name with respect to package root
    pub description: Option<String>,
    #[serde(rename = "cr-number")]
    pub cr_number: usize,
}

pub(crate) async fn get_cr_about(
    config: &fpm::Config,
    cr_number: usize,
) -> fpm::Result<fpm::cr::CRAbout> {
    let cr_about_path = config.cr_path(cr_number).join("-/about.ftd");
    if !cr_about_path.exists() {
        // TODO: should we error out here?
        return Ok(Default::default());
    }

    let doc = std::fs::read_to_string(&cr_about_path)?;
    resolve_cr_about(&doc).await
}

pub(crate) async fn resolve_cr_about(content: &str) -> fpm::Result<fpm::cr::CRAbout> {
    if content.trim().is_empty() {
        return Err(fpm::Error::UsageError {
            message: "Content is empty in cr about".to_string(),
        });
    }
    let lib = fpm::FPMLibrary::default();
    let b = match fpm::doc::parse_ftd(".about.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse .latest.ftd: {:?}", &e);
            todo!();
        }
    };
    Ok(b.get("fpm#cr-about")?)
}

pub(crate) async fn create_cr_about(
    config: &fpm::Config,
    cr_about: &fpm::cr::CRAbout,
) -> fpm::Result<()> {
    let about_content = generate_cr_about_content(cr_about);
    fpm::utils::update(
        &config.cr_path(cr_about.cr_number),
        "-/about.ftd",
        about_content.as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) fn generate_cr_about_content(cr_about: &fpm::cr::CRAbout) -> String {
    let mut about_content = format!(
        "-- import: fpm\n\n\n-- fpm.cr-about: {}\ncr-number: {}",
        cr_about.title, cr_about.cr_number
    );
    if let Some(ref description) = cr_about.description {
        about_content = format!("{}\n\n{}", about_content, description);
    }
    about_content
}
