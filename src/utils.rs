pub fn process_dir<F>(
    directory: String,
    depth: usize,
    base_path: String,
    ignore_dir: &[&str],
    f: &F,
) -> u32
where
    F: Fn(&str, String, String, usize),
{
    let mut count: u32 = 0;
    for entry in std::fs::read_dir(&directory).expect("Panic! Unable to process the directory") {
        let e = entry.expect("Panic: Doc not found");
        let md = std::fs::metadata(e.path()).expect("Doc Metadata evaluation failed");
        let doc_path = e
            .path()
            .to_str()
            .expect("Directory path is expected")
            .to_string();

        if depth == 0 && doc_path.as_str().ends_with("FPM.ftd") {
            // pass the FPM.ftd file at the base level
        } else if md.is_dir() {
            // Iterate the children
            let id = doc_path.split('/').last();
            if id.is_some() && ignore_dir.contains(&id.unwrap()) {
                continue;
            }
            count += process_dir(
                doc_path,
                depth + 1,
                base_path.as_str().to_string(),
                ignore_dir,
                f,
            );
        } else if doc_path.as_str().ends_with(".ftd") {
            // process the document
            let doc = std::fs::read_to_string(doc_path).expect("cant read file");
            let id = e.path().clone();
            let id = id.to_str().expect(">>>").split('/');
            let len = id.clone().count();

            f(
                id.skip(len - (depth + 1))
                    .take_while(|_| true)
                    .collect::<Vec<&str>>()
                    .join("/")
                    .as_str(),
                doc,
                base_path.as_str().to_string(),
                depth,
            );
            count += 1;
        }
    }
    count
}
