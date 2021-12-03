pub async fn sync() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    std::fs::create_dir_all(format!("{}/.history", config.root.as_str()).as_str())
        .expect("failed to create build folder");

    let snapshots = get_latest_snapshots(config.root.as_str())?;

    let timestamp = fpm::get_timestamp_nanosecond();
    let mut modified_files = vec![];

    let mut snapshot_data = "".to_string();
    for doc in fpm::process_dir(config.root.as_str()).await? {
        if let Some((snapshot, is_modified)) = write(&doc, timestamp, &snapshots).await? {
            if is_modified {
                modified_files.push(snapshot.file.to_string());
            }
            snapshot_data = format!(
                "{}\n\n-- fpm.snapshots: {}\ntimestamp: {}",
                snapshot_data, snapshot.file, snapshot.timestamp
            );
        }
    }

    create_latest_snapshots(config.root.as_str(), &snapshot_data)?;

    if modified_files.is_empty() {
        println!("Everything is upto date.");
    } else {
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
    snapshots: &[Snapshot],
) -> fpm::Result<Option<(Snapshot, bool)>> {
    use tokio::io::AsyncWriteExt;

    if doc.id.starts_with(".history") {
        return Ok(None);
    }

    if doc.id.contains('/') {
        let (dir, _) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))?;
    }

    /*let (path, doc_name) = if doc.id.contains('/') {
        let (dir, doc_name) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))?;
        (
            format!("{}/.history/{}", doc.base_path.as_str(), dir),
            doc_name.to_string(),
        )
    } else {
        (
            format!("{}/.history", doc.base_path.as_str()),
            doc.id.to_string(),
        )
    };

    let mut files = tokio::fs::read_dir(&path).await?;

    let mut max_timestamp: Option<(String, String)> = None;
    while let Some(n) = files.next_entry().await? {
        let p = format!("{}/{}.", path, doc_name.replace(".ftd", ""));
        let file = n.path().to_str().unwrap().to_string();
        if file.starts_with(&p) {
            let timestamp = file
                .replace(&format!("{}/{}.", path, doc_name.replace(".ftd", "")), "")
                .replace(".ftd", "");
            if let Some((t, _)) = &max_timestamp {
                if *t > timestamp {
                    continue;
                }
            }
            max_timestamp = Some((timestamp, file.to_string()));
        }
    }

    if let Some((timestamp, path)) = max_timestamp {
        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.document.eq(&existing_doc) {
            return Ok(Some((
                Snapshot {
                    file: doc_name.to_string(),
                    timestamp,
                },
                false,
            )));
        }
    }*/

    for snapshot in snapshots {
        if doc.id.eq(&snapshot.file) {
            let path = format!(
                "{}/.history/{}",
                doc.base_path.as_str(),
                doc.id
                    .replace(".ftd", &format!(".{}.ftd", &snapshot.timestamp))
            );

            let existing_doc = tokio::fs::read_to_string(&path).await?;
            if doc.document.eq(&existing_doc) {
                return Ok(Some((
                    Snapshot {
                        file: doc.id.to_string(),
                        timestamp: snapshot.timestamp.to_string(),
                    },
                    false,
                )));
            }
            break;
        }
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;
    f.write_all(doc.document.as_bytes()).await?;

    Ok(Some((
        Snapshot {
            file: doc.id.to_string(),
            timestamp: timestamp.to_string(),
        },
        true,
    )))
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

fn get_latest_snapshots(base_path: &str) -> fpm::Result<Vec<Snapshot>> {
    let new_file_path = format!("{}/.history/.latest.ftd", base_path);
    if std::fs::metadata(&new_file_path).is_err() {
        create_latest_snapshots(base_path, "")?;
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
    Snapshot::parse(&b)
}

fn create_latest_snapshots(base_path: &str, data: &str) -> fpm::Result<()> {
    use std::io::Write;

    let new_file_path = format!("{}/.history/.latest.ftd", base_path);
    let snapshot_data = format!("-- import: fpm{}", data);

    let mut f = std::fs::File::create(new_file_path.as_str())?;

    f.write_all(snapshot_data.as_bytes())?;

    Ok(())
}
