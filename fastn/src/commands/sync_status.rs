pub const COMMAND: &str = "sync-status";

pub fn command() -> clap::Command {
    clap::Command::new(COMMAND)
        .about("Show the sync status of files in this fastn package")
        .arg(clap::arg!(file: <FILE>... "The file(s) to see status of (leave empty to see status of entire package)").required(false))
        .hide(true) // hidden since the feature is not being released yet.
}

pub async fn handle_command(matches: &clap::ArgMatches) -> fastn::Result<()> {
    use fastn::utils::ValueOf;

    sync_status(
        &fastn::Config::read(None, true, None).await?,
        matches.value_of_("file"), // TODO: handle multiple files
    )
    .await
}

async fn sync_status(config: &fastn::Config, source: Option<&str>) -> fastn::Result<()> {
    use itertools::Itertools;

    let get_files_status = config.get_files_status().await?;
    if let Some(source) = source {
        if let Some(file_status) = get_files_status
            .iter()
            .find(|v| v.get_file_path().eq(source))
        {
            print_status(file_status, true);
            return Ok(());
        }
        return Err(fastn::Error::UsageError {
            message: format!("{} not found", source),
        });
    }
    get_files_status
        .iter()
        .map(|v| print_status(v, false))
        .collect_vec();
    Ok(())
}

pub(crate) fn print_status(file_status: &fastn::sync_utils::FileStatus, print_untracked: bool) {
    let (file_status, path, status) = match file_status {
        fastn::sync_utils::FileStatus::Add { path, status, .. } => ("Added", path, status),
        fastn::sync_utils::FileStatus::Update { path, status, .. } => ("Updated", path, status),
        fastn::sync_utils::FileStatus::Delete { path, status, .. } => ("Deleted", path, status),
        fastn::sync_utils::FileStatus::Uptodate { path, .. } => {
            if print_untracked {
                println!("Up-to-date: {}", path);
            }
            return;
        }
    };
    match status {
        fastn::sync_utils::Status::Conflict(_) => println!("Conflicted: {}", path),
        fastn::sync_utils::Status::CloneEditedRemoteDeleted(_) => {
            println!("CloneEditedRemoteDeleted: {}", path)
        }
        fastn::sync_utils::Status::CloneDeletedRemoteEdited(_) => {
            println!("CloneDeletedRemoteEdited: {}", path)
        }
        fastn::sync_utils::Status::NoConflict => println!("{}: {}", file_status, path),
        fastn::sync_utils::Status::CloneAddedRemoteAdded(_) => {
            println!("CloneAddedRemoteAdded: {}", path)
        }
    }
}