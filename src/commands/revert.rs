use itertools::Itertools;

pub async fn revert(config: &fpm::Config, path: &str, root: Option<String>) -> fpm::Result<()> {
    if let Some((cr_number, path)) = fpm::cr::get_cr_and_path_from_id(path, &root) {
        let mut workspaces = fpm::snapshot::get_cr_workspace(config, cr_number).await?;
        revert_cr_(config, &path, &mut workspaces, cr_number).await?;
        if !workspaces.is_empty() {
            fpm::snapshot::create_workspace(
                &config.cr_path(cr_number),
                workspaces.into_values().collect_vec().as_slice(),
            )
            .await?;
        }
        return Ok(());
    }

    let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    revert_(config, path, &mut workspaces, &snapshots).await?;
    if !workspaces.is_empty() {
        fpm::snapshot::create_workspace(
            &config.root,
            workspaces.into_values().collect_vec().as_slice(),
        )
        .await?;
    }
    Ok(())
}

pub(crate) async fn revert_(
    config: &fpm::Config,
    path: &str,
    workspaces: &mut std::collections::BTreeMap<String, fpm::snapshot::Workspace>,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fpm::Result<()> {
    if let Some(workspace) = workspaces.get_mut(path) {
        if workspace
            .workspace
            .eq(&fpm::snapshot::WorkspaceType::ClientEditedServerDeleted)
        {
            if config.root.join(path).exists() {
                tokio::fs::remove_file(config.root.join(path)).await?;
            }
        } else {
            let revert_path =
                fpm::utils::history_path(path, config.root.as_str(), &workspace.conflicted);
            tokio::fs::copy(revert_path, config.root.join(path)).await?;
        }
        workspace.set_revert();
    } else if let Some(timestamp) = snapshots.get(path) {
        let revert_path = fpm::utils::history_path(path, config.root.as_str(), timestamp);

        fpm::utils::update(
            &config.root,
            path,
            tokio::fs::read(revert_path).await?.as_slice(),
        )
        .await?;
    } else {
        tokio::fs::remove_file(config.root.join(path)).await?;
    }

    Ok(())
}

pub(crate) async fn revert_cr_(
    config: &fpm::Config,
    path: &str,
    workspaces: &mut std::collections::BTreeMap<String, fpm::snapshot::Workspace>,
    cr_number: usize,
) -> fpm::Result<()> {
    let mut cr_delete = fpm::cr::get_cr_delete(config, cr_number).await?;
    let mut cr_track = fpm::tracker::get_cr_track(config, cr_number).await?;

    cr_delete = cr_delete.into_iter().filter(|(v, _)| !v.eq(path)).collect();
    cr_track = cr_track.into_iter().filter(|(v, _)| !v.eq(path)).collect();

    fpm::cr::create_cr_delete(
        config,
        cr_delete.into_values().collect_vec().as_slice(),
        cr_number,
    )
    .await?;
    fpm::tracker::create_cr_track(
        config,
        cr_track.into_values().collect_vec().as_slice(),
        cr_number,
    )
    .await?;

    let root = config.cr_path(cr_number);
    if let Some(workspace) = workspaces.get_mut(path) {
        workspace.set_revert();
    }
    if root.join(path).exists() {
        tokio::fs::remove_file(root.join(path)).await?;
    }

    Ok(())
}
