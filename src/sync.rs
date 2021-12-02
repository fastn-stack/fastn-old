pub fn sync() {
    let (_fpm_config, base_dir) = fpm::check();

    std::fs::create_dir_all(format!("{}/.history", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    fpm::process_dir(
        base_dir.clone(),
        0,
        base_dir,
        &[".history", ".build"],
        &write,
    );
}

fn write(id: &str, doc: String, base_path: String, _depth: usize) {
    use chrono::prelude::*;
    use std::io::Write;

    if id.starts_with(".history") {
        return;
    }

    if id.contains('/') {
        let (dir, _) = id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", base_path.as_str(), dir))
            .expect("failed to create directory folder for doc");
    }

    let new_file_path = format!(
        "{}/.history/{}",
        base_path.as_str(),
        id.replace(".ftd", &format!(".{}.ftd", Utc::now().timestamp()))
    );

    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    f.write_all(doc.as_bytes())
        .expect("failed to write to .html file");
    println!("Generated history [{}]", format!("{}/{}", base_path, id));
}
