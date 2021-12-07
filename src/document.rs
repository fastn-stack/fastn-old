#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

pub(crate) async fn process_dir(directory: &str) -> fpm::Result<Vec<Document>> {
    let mut documents: Vec<Document> = vec![];

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
        // TODO: Make this concurrent async
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

    // #[async_recursion::async_recursion]
    // async fn process_dir_(
    //     documents: &mut Vec<Document>,
    //     directory: &std::path::Path,
    //     depth: usize,
    //     base_path: &std::path::Path,
    // ) -> fpm::Result<()> {
    //     let mut r = tokio::fs::read_dir(&directory).await?;
    //     while let Some(entry) = r.next_entry().await? {
    //         let doc_path = entry.path();
    //         let md = tokio::fs::metadata(&doc_path).await?;

    //         if md.is_dir() {
    //             // Iterate the children
    //             let id = doc_path.to_str().unwrap_or_default().split('/').last();
    //             if id.is_some() && [".history", ".build", ".packages"].contains(&id.unwrap()) {
    //                 // ignore .history and .build directory
    //                 continue;
    //             }
    //             process_dir_(documents, &doc_path, depth + 1, base_path).await?;
    //         } else if doc_path.to_str().unwrap_or_default().ends_with(".ftd") {
    //             // process the document
    //             let doc = tokio::fs::read_to_string(&doc_path).await?;
    //             let id = doc_path.to_str().unwrap_or_default().split('/');
    //             let len = id.clone().count();

    //             documents.push(Document {
    //                 id: id
    //                     .skip(len - (depth + 1))
    //                     .take_while(|_| true)
    //                     .collect::<Vec<&str>>()
    //                     .join("/")
    //                     .to_string(),
    //                 document: doc,
    //                 base_path: base_path.to_str().unwrap_or_default().to_string(),
    //                 depth,
    //             });
    //         }
    //     }
    //     Ok(())
    // }
}
