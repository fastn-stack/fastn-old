#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
    pub user_id: String,
}
pub async fn matched_identities(
    ud: UserDetail,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    let instagram_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("instagram"))
        .collect::<Vec<&fpm::user_group::UserIdentity>>();

    if instagram_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];

    Ok(matched_identities)
}
