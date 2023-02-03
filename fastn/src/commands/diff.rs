pub async fn diff(
    config: &fastn::Config,
    files: Option<Vec<String>>,
    all: bool,
) -> fastn::Result<()> {
    let snapshots = fastn::snapshot::get_latest_snapshots(&config.root).await?;
    let all = all || files.is_some();
    let documents = if let Some(ref files) = files {
        let files = files
            .iter()
            .map(|x| config.root.join(x))
            .collect::<Vec<camino::Utf8PathBuf>>();
        fastn::paths_to_files(config.package.name.as_str(), files, config.root.as_path()).await?
    } else {
        config.get_files(&config.package).await?
    };
    for doc in documents {
        if let Some(diff) = get_diffy(&doc, &snapshots).await? {
            println!("diff: {}", doc.get_id());
            println!("{}", diff);
        }
        if all {
            get_track_diff(&doc, &snapshots, config.root.as_str()).await?;
        }
    }
    Ok(())
}

async fn get_diffy(
    doc: &fastn::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fastn::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fastn::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);
        let content = tokio::fs::read_to_string(&doc.get_full_path()).await?;

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if content.eq(&existing_doc) {
            return Ok(None);
        }
        let patch = diffy::create_patch(&existing_doc, &content);
        let diff = diffy::PatchFormatter::new()
            .with_color()
            .fmt_patch(&patch)
            .to_string();
        return Ok(Some(diff));
    }
    Ok(None)
}

async fn get_track_diff(
    doc: &fastn::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
    base_path: &str,
) -> fastn::Result<()> {
    let path = fastn::utils::track_path(&doc.get_id(), &doc.get_base_path());
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let tracks = fastn::tracker::get_tracks(base_path, &path)?;
    for track in tracks.values() {
        if let Some(timestamp) = snapshots.get(&track.filename) {
            if track.other_timestamp.is_none() {
                continue;
            }
            let now_path =
                fastn::utils::history_path(&track.filename, &doc.get_base_path(), timestamp);

            let then_path = fastn::utils::history_path(
                &track.filename,
                &doc.get_base_path(),
                track.other_timestamp.as_ref().unwrap(),
            );

            let now_doc = tokio::fs::read_to_string(&now_path).await?;
            let then_doc = tokio::fs::read_to_string(&then_path).await?;
            if now_doc.eq(&then_doc) {
                continue;
            }
            let patch = diffy::create_patch(&then_doc, &now_doc);
            let diff = diffy::PatchFormatter::new()
                .with_color()
                .fmt_patch(&patch)
                .to_string();
            println!(
                "diff {} -> {}: {}",
                doc.get_id(),
                then_path
                    .to_string()
                    .replace(&format!("{}/.history/", doc.get_base_path()), ""),
                now_path
                    .to_string()
                    .replace(&format!("{}/.history/", doc.get_base_path()), ""),
            );
            println!("{}", diff);
        }
    }
    Ok(())
}
