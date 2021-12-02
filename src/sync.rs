pub async fn sync() {
    let (_fpm_config, base_dir) = fpm::check().await;

    std::fs::create_dir_all(format!("{}/.history", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    let timestamp = fpm::get_timestamp_nanosecond();
    for doc in fpm::process_dir(base_dir.clone(), 0, base_dir, &[]) {
        write(&doc, timestamp);
    }
}

fn write(doc: &fpm::Document, timestamp: u128) {
    use std::io::Write;

    if doc.id.starts_with(".history") {
        return;
    }

    let (path, doc_name) = if doc.id.contains('/') {
        let (dir, doc_name) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))
            .expect("failed to create directory folder for doc");
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

    let files = std::fs::read_dir(&path).expect("Panic! Unable to process the directory");

    let mut max_timestamp: Option<(String, String)> = None;
    for n in files.flatten() {
        let p = format!(r"{}/{}\.\d+\.ftd", path, doc_name.replace(".ftd", ""));
        let regex = regex::Regex::new(&p).unwrap();
        let file = n.path().to_str().unwrap().to_string();
        if regex.is_match(&file) {
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

    if let Some((_, path)) = max_timestamp {
        let existing_doc = std::fs::read_to_string(&path).expect("cant read file");
        if doc.document.eq(&existing_doc) {
            return;
        }
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    f.write_all(doc.document.as_bytes())
        .expect("failed to write to .html file");
    println!(
        "Generated history [{}]",
        format!("{}/{}", doc.base_path, doc.id)
    );
}
