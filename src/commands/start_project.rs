async fn template_contents(project_name: &str) -> (String, String) {
    let ftd = format!("-- import: fpm\n\n-- fpm.package: {}", project_name);
    let index = "-- ftd.text: Hello world".to_string();

    (ftd, index)
}

async fn write_file(file_name: &str, dir: &str, content: &str) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;
    let file_path = format!("{}/{}", dir, file_name);
    let mut fp = tokio::fs::File::create(file_path).await?;
    fp.write_all(content.as_bytes()).await?;
    Ok(())
}

pub async fn start_project(name: Option<&str>, path: Option<&str>) -> fpm::Result<()> {
    let base_path = std::env::current_dir()?.to_str().unwrap().to_string();
    // Not using config for base path as it requires manifest or FPM.ftd file for building and will throw error
    // and since this command should work from anywhere within the system
    // so we dont need to rely on config for using it

    // name is a required field so it will always be some defined string (cant be None)
    // path is an optional field and has a default value "." for current working directory (also cant be None)
    let (name, path) = (name.unwrap(), path.unwrap());
    let final_dir = match path {
        "." => format!("{}/{}", base_path, name),
        _ => format!("{}/{}/{}", base_path, path, name),
    };

    // Create all directories if not present
    tokio::fs::create_dir_all(final_dir.as_str()).await?;

    let tmp_contents = template_contents(name).await;
    let tmp_fpm = tmp_contents.0;
    let tmp_index = tmp_contents.1;

    write_file("FPM.ftd", &final_dir, &tmp_fpm).await?;
    write_file("index.ftd", &final_dir, &tmp_index).await?;
    println!(
        "Template FTD project created - {}\nPath -{}",
        name, final_dir
    );

    Ok(())
}
