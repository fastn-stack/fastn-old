pub async fn sync() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    std::fs::create_dir_all(format!("{}/.history", config.root.as_str()).as_str())
        .expect("failed to create build folder");

    let snapshots = get_latest_snapshots(config.root.as_str())?;

    let timestamp = fpm::get_timestamp_nanosecond();
    let mut modified_files = vec![];
    let mut new_snapshots = vec![];
    for doc in fpm::process_dir(config.root.as_str()).await? {
        if doc.id.starts_with(".history") {
            continue;
        }
        let (snapshot, is_modified) = write(&doc, timestamp, &snapshots).await?;
        if is_modified {
            modified_files.push(snapshot.file.to_string());
        }
        new_snapshots.push(snapshot);
    }

    if modified_files.is_empty() {
        println!("Everything is upto date.");
    } else {
        create_latest_snapshots(config.root.as_str(), &new_snapshots).await?;
        println!(
            "Repo for {} is github, directly syncing with .history.",
            config.package.name
        );
        for file in modified_files {
            println!("{}", file);
        }
    }
    Ok(())
}

async fn write(
    doc: &fpm::Document,
    timestamp: u128,
    snapshots: &std::collections::BTreeMap<String, String>,
) -> fpm::Result<(Snapshot, bool)> {
    use tokio::io::AsyncWriteExt;

    if doc.id.contains('/') {
        let (dir, _) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))?;
    }

    if let Some(timestamp) = snapshots.get(&doc.id) {
        let path = format!(
            "{}/.history/{}",
            doc.base_path.as_str(),
            doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
        );

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.document.eq(&existing_doc) {
            return Ok((
                Snapshot {
                    file: doc.id.to_string(),
                    timestamp: timestamp.to_string(),
                },
                false,
            ));
        }
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;
    f.write_all(doc.document.as_bytes()).await?;

    Ok((
        Snapshot {
            file: doc.id.to_string(),
            timestamp: timestamp.to_string(),
        },
        true,
    ))
}

#[derive(serde::Deserialize, Debug)]
pub struct Snapshot {
    pub file: String,
    pub timestamp: String,
}

impl Snapshot {
    pub fn parse(b: &ftd::p2::Document) -> fpm::Result<Vec<Snapshot>> {
        Ok(b.to_owned().get::<Vec<Snapshot>>("fpm#snapshots")?)
    }
}

fn get_latest_snapshots(
    base_path: &str,
) -> fpm::Result<std::collections::BTreeMap<String, String>> {
    let mut snapshots = std::collections::BTreeMap::new();
    let new_file_path = format!("{}/.history/.latest.ftd", base_path);
    if std::fs::metadata(&new_file_path).is_err() {
        return Ok(snapshots);
    }

    let lib = fpm::Library {};
    let doc = std::fs::read_to_string(&new_file_path)?;
    let b = match ftd::p2::Document::from(base_path, doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", base_path, &e);
            todo!();
        }
    };
    let snapshot_list = Snapshot::parse(&b)?;
    for snapshot in snapshot_list {
        snapshots.insert(snapshot.file, snapshot.timestamp);
    }
    Ok(snapshots)
}

async fn create_latest_snapshots(base_path: &str, snapshots: &[Snapshot]) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let new_file_path = format!("{}/.history/.latest.ftd", base_path);
    let mut snapshot_data = "-- import: fpm".to_string();

    for snapshot in snapshots {
        snapshot_data = format!(
            "{}\n\n-- fpm.snapshots: {}\ntimestamp: {}",
            snapshot_data, snapshot.file, snapshot.timestamp
        );
    }

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    f.write_all(snapshot_data.as_bytes()).await?;

    Ok(())
}
