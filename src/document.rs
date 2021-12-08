#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

pub(crate) async fn process_dir(directory: &str) -> fpm::Result<Vec<Document>> {
    let mut documents: Vec<Document> = vec![];
    // TODO: Get this concurrent async to work
    // let all_files = ignore::Walk::new(directory.to_string())
    //     .into_iter()
    //     .map(|x| {
    //         tokio::spawn(process_file_(
    //             &mut documents,
    //             x.unwrap().into_path(),
    //             directory,
    //         ))
    //     })
    //     .collect::<Vec<tokio::task::JoinHandle<fpm::Result<()>>>>();
    // futures::future::join_all(all_files).await;

    for x in ignore::Walk::new(directory.to_string()) {
        process_file_(&mut documents, x.unwrap().into_path(), directory).await?;
    }
    documents.sort_by_key(|v| v.id.clone());

    return Ok(documents);

    async fn process_file_(
        documents: &mut Vec<Document>,
        doc_path: std::path::PathBuf,
        dir: &str,
    ) -> fpm::Result<()> {
        if !&doc_path.is_dir() {
            let doc_path_str = doc_path.to_str().unwrap();
            let doc = tokio::fs::read_to_string(&doc_path);
            if let Some((_base_path, id)) = std::fs::canonicalize(&doc_path)?
                .to_str()
                .unwrap()
                .rsplit_once(format!("{}/", dir).as_str())
            {
                documents.push(Document {
                    id: id.to_string(),
                    document: doc.await?,
                    base_path: dir.to_string(),
                    depth: doc_path_str.split('/').count() - 1,
                });
            }
        }
        Ok(())
    }
}
