pub async fn diff(config: &fpm::Config) -> fpm::Result<()> {
    let snapshots = fpm::snapshot::get_latest_snapshots(config).await?;
    for doc in fpm::get_documents(config).await? {
        if let Some(diff) = get_diffy(&doc, &snapshots).await? {
            println!("diff: {}", doc.get_id());
            println!("{}", diff);
        }
        get_track_diff(&doc, &snapshots, config.root.as_str()).await?;
    }
    Ok(())
}

async fn get_diffy(
    doc: &fpm::File,
    snapshots: &std::collections::BTreeMap<String, String>,
) -> fpm::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fpm::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);
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
    doc: &fpm::File,
    snapshots: &std::collections::BTreeMap<String, String>,
    base_path: &str,
) -> fpm::Result<()> {
    let path = fpm::utils::track_path(&doc.get_id(), &doc.get_base_path());
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let tracks = fpm::tracker::get_tracks(base_path, &path)?;
    for track in tracks.values() {
        if let Some(timestamp) = snapshots.get(&track.document_name) {
            if track.other_timestamp.is_none() {
                continue;
            }
            let now_path =
                fpm::utils::history_path(&track.document_name, &doc.get_base_path(), timestamp);

            let then_path = fpm::utils::history_path(
                &track.document_name,
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
