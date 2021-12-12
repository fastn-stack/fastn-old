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
        let file_extension = if let Some((_, b)) = doc.get_id().rsplit_once('.') {
            Some(b.to_string())
        } else {
            None
        };
        let path = format!("{}/.history/{}", doc.get_base_path(), {
            if let Some(ref ext) = file_extension {
                doc.get_id()
                    .replace(&format!(".{}", ext), &format!(".{}.{}", timestamp, ext))
            } else {
                format!(".{}", timestamp)
            }
        });

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
    let path = format!(
        "{}/.tracks/{}",
        doc.get_base_path().as_str(),
        format!("{}.track", doc.get_id())
    );
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let tracks = fpm::tracker::get_tracks(base_path, &path)?;
    for track in tracks.values() {
        if let Some(timestamp) = snapshots.get(&track.document_name) {
            if track.other_timestamp.is_none() {
                continue;
            }
            let file_extension = if let Some((_, b)) = track.document_name.rsplit_once('.') {
                Some(b.to_string())
            } else {
                None
            };
            let now_path = format!("{}/.history/{}", doc.get_base_path(), {
                if let Some(ref ext) = file_extension {
                    track
                        .document_name
                        .replace(&format!(".{}", ext), &format!(".{}.{}", timestamp, ext))
                } else {
                    format!(".{}", timestamp)
                }
            });
            let then_path = format!("{}/.history/{}", doc.get_base_path(), {
                if let Some(ref ext) = file_extension {
                    track.document_name.replace(
                        &format!(".{}", ext),
                        &format!(".{}.{}", track.other_timestamp.as_ref().unwrap(), ext),
                    )
                } else {
                    format!(".{}", timestamp)
                }
            });
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
                {
                    if let Some(ref ext) = file_extension {
                        track.document_name.replace(
                            &format!(".{}", ext),
                            &format!(".{}.{}", track.other_timestamp.as_ref().unwrap(), ext),
                        )
                    } else {
                        format!(".{}", timestamp)
                    }
                },
                {
                    if let Some(ref ext) = file_extension {
                        track
                            .document_name
                            .replace(&format!(".{}", ext), &format!(".{}.{}", timestamp, ext))
                    } else {
                        format!(".{}", timestamp)
                    }
                }
            );
            println!("{}", diff);
        }
    }
    Ok(())
}
