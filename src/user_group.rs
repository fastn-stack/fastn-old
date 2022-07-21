// identities to group, test also
#[derive(Debug, Clone)]
pub struct UserGroup {
    pub title: Option<String>,
    pub id: String,
    pub identities: Vec<(String, String)>,
    pub excluded_identities: Vec<(String, String)>,
    pub import: Vec<String>,
}

// This type is needed to deserialize ftd to rust

#[derive(serde::Deserialize, Debug)]
pub struct UserGroupTemp {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub import: Vec<String>,
    pub email: Vec<String>,
    #[serde(rename = "-email")]
    pub excluded_email: Vec<String>,
    pub github: Vec<String>,
    #[serde(rename = "-github")]
    pub excluded_github: Vec<String>,
    pub telegram: Vec<String>,
    #[serde(rename = "-telegram")]
    pub excluded_telegram: Vec<String>,
}

impl UserGroupTemp {
    pub fn to_user_group(self) -> UserGroup {
        let identities = {
            let mut identities = vec![];
            identities.extend(
                self.email
                    .into_iter()
                    .map(|identity| ("email".to_string(), identity)),
            );

            identities.extend(
                self.github
                    .into_iter()
                    .map(|identity| ("github".to_string(), identity)),
            );

            identities.extend(
                self.telegram
                    .into_iter()
                    .map(|identity| ("telegram".to_string(), identity)),
            );
            identities
        };

        let excluded_identities = {
            let mut excluded_identities = vec![];
            excluded_identities.extend(
                self.excluded_email
                    .into_iter()
                    .map(|identity| ("email".to_string(), identity)),
            );

            excluded_identities.extend(
                self.excluded_github
                    .into_iter()
                    .map(|identity| ("github".to_string(), identity)),
            );

            excluded_identities.extend(
                self.excluded_telegram
                    .into_iter()
                    .map(|identity| ("telegram".to_string(), identity)),
            );
            excluded_identities
        };

        UserGroup {
            id: self.id,
            identities,
            excluded_identities,
            title: self.title,
            import: self.import,
        }
    }
}
