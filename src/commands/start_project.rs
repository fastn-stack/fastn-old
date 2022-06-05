use std::fs;
use std::env;
use std::fs::File;
use std::io::prelude::*;

async fn template_contents(project_name: &str) -> (String,String){
	let ftd = format!("-- import: fpm\n\n-- fpm.package: {}", project_name);
	let index = format!("-- ftd.text: Hello world");

	(ftd,index)
}

async fn write_file(file_name: &str, dir: &str, content: &str){
	let file_path = format!("{}/{}",dir,file_name);
	let mut fp = File::create(file_path)
		.expect("Could not create file!");

	fp.write_all(content.as_bytes())
		.expect("Could not write contents into the file!");
}
pub async fn start_project(name: Option <&str>,
						   path: Option <&str>) -> fpm::Result<()>{
	/*
    if path != '.' (will be some user input)
    final_dir = base_path/relative_path/project_folder => template-files

    if path == '.' (Default value being the current directory)
    final_dir = base_path/project_folder => template-files
	*/
	let base_path = env::current_dir()?.to_str().unwrap().to_string();
	/* Not using config for base path as it throws error and requires manifest or FPM.ftd file
	   and since this command should work from anywhere within the system
	   so we dont need any other file dependency for using it
	*/
	// let base_path = config.root.as_str();

	let mut final_dir = "".to_string();
	if let Some(relative_path) = path{
		match relative_path {
			"." => final_dir = format!("{}/{}", base_path, name.unwrap()),
			_ => final_dir = format!("{}/{}/{}", base_path, relative_path, name.unwrap())
		}
	}
	// Create all directories if not present
	fs::create_dir_all(final_dir.as_str())?;

	let tmp_contents = template_contents(name.unwrap()).await;
	let tmp_fpm = tmp_contents.0;
	let tmp_index = tmp_contents.1;

	write_file("FPM.ftd", final_dir.as_str(), &tmp_fpm).await;
	write_file("index.ftd", final_dir.as_str(), &tmp_index).await;
	println!("Template FTD project created - {}\nPath -{}",name.unwrap(),final_dir);

	Ok(())
}