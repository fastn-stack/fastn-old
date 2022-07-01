// New cache
// functions get, update, update and get, get and update, get or init
// all should be atomic operation

/// For Now we can get file name update the value

// TODO: Need to change it later

async fn get(path: &str) -> fpm::Result<usize> {
    let value = tokio::fs::read_to_string(path).await?;
    Ok(value.parse()?)
}

async fn create(path: &str) -> fpm::Result<usize> {
    use tokio::io::AsyncWriteExt;
    let content = 1;
    tokio::fs::File::create(path)
        .await?
        .write_all(content.to_string().as_bytes())
        .await?;
    let value = tokio::fs::read_to_string(path).await?;
    Ok(value.parse()?)
}
