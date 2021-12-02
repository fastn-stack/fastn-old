pub fn build() {
    let (_fpm_config, base_dir) = fpm::check();

    std::fs::create_dir_all(format!("{}/.build", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    fpm::process_dir(
        base_dir.clone(),
        0,
        base_dir,
        &[".history", ".build"],
        &write,
    );
}

fn write(id: &str, doc: String, base_path: String, depth: usize) {
    use std::io::Write;

    let lib = fpm::Library {};
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };
    if !(depth == 0 && id == "index.ftd") {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            base_path.as_str(),
            id.replace(".ftd", "")
        ))
        .expect("failed to create directory folder for doc");
    }
    let new_file_path = format!(
        "{}/.build/{}",
        base_path.as_str(),
        if id == "index.ftd" {
            "index.html".to_string()
        } else {
            id.replace(".ftd", "/index.html")
        }
    );
    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    let doc = b.to_rt("main", id);

    f.write_all(
        ftd::html()
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace("__ftd__", b.html("main", id).as_str())
            .replace("__ftd_js__", ftd::js())
            .as_bytes(),
    )
    .expect("failed to write to .html file");
    println!(
        "Generated {} [{}]",
        new_file_path,
        format!("{}/{}", base_path, id)
    );
}
