use itertools::Itertools;

// identities to group, test also
#[derive(Debug, Clone)]
pub struct UserGroup {
    pub title: Option<String>,
    pub id: String,
    pub identities: Vec<(String, String)>,
    pub excluded_identities: Vec<(String, String)>,

    /// if package name is abrark.com and it has user-group with id my-all-readers
    /// so import string will be abrark.com/my-all-readers
    pub import: Vec<String>,
    pub description: Option<String>,
}

// This type is needed to deserialize ftd to rust
#[derive(serde::Deserialize, Debug)]
pub struct UserGroupTemp {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,

    /// if package name is abrark.com and it has user-group with id my-all-readers
    /// so import string will be abrark.com/my-all-readers
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

/*
.ftd code
-- fpm.user-group list email-groups:
$processor$: user-groups

-- fpm.user-group list email-groups:
$processor$: user-groups
 */

#[derive(Debug, serde::Serialize)]
pub struct UserGroupCompat {
    id: String,
    title: Option<String>,
    description: Option<String>,
    // All Member(with import) - excluded members
    #[serde(rename = "group-members")]
    group_members: Vec<fpm::library::full_sitemap::KeyValueData>,
}

impl UserGroup {
    pub fn to_group_compat(&self, config: &fpm::Config) -> UserGroupCompat {
        // TODO:
        // Main logic is group_members = all_group(identities) - all_group(excluded_identities)
        // Combine all imported group identities and then exclude all group identities

        // for import_identity in self.import.iter() {
        //     let (package, identity) =
        //         import_identity
        //             .rsplit_once('/')
        //             .ok_or_else(|| ftd::p1::Error::ParseError {
        //                 message: format!(
        //                     "import_identity: {}, does not contain `/`",
        //                     import_identity
        //                 ),
        //                 doc_id: "FPM.ftd".to_string(),
        //                 line_number: 0,
        //             })?;
        //     // either http of file-system
        //     // config.package.fs_fetch_by_file_name()
        //     // config.packages_root.join(package, "")
        // }

        let excluded_identities: std::collections::HashMap<&str, &str> = self
            .excluded_identities
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let is_excluded =
            |key: &str, value: &str| excluded_identities.get(key).map_or(false, |v| v.eq(&value));

        UserGroupCompat {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            group_members: self
                .identities
                .iter()
                .filter(|(k, v)| !is_excluded(k.as_str(), v.as_str()))
                .map(|(key, value)| fpm::library::full_sitemap::KeyValueData {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect_vec(),
        }
    }
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
            description: self.description,
            identities,
            excluded_identities,
            title: self.title,
            import: self.import,
        }
    }
}

pub mod processor {
    use itertools::Itertools;

    pub fn user_groups(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let g = config
            .groups
            .iter()
            .map(|(_, g)| g.to_group_compat(config))
            .collect_vec();
        doc.from_json(&g, section)
    }

    pub fn user_group_by_id(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let id = section.header.str(doc.name, section.line_number, "id")?;
        let g = config
            .groups
            .get(id)
            .map(|g| g.to_group_compat(config))
            .ok_or_else(|| ftd::p1::Error::NotFound {
                key: format!("user-group: `{}` not found", id),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            })?;
        doc.from_json(&g, section)
    }
}
