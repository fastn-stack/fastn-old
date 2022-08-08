// identities to group, test also
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserIdentity {
    pub key: String,
    pub value: String,
}

impl UserIdentity {
    pub fn from(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

impl ToString for UserIdentity {
    fn to_string(&self) -> String {
        format!("{}: {}", self.key, self.value)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserGroup {
    pub title: Option<String>,
    pub id: String,
    pub identities: Vec<UserIdentity>,
    pub excluded_identities: Vec<UserIdentity>,

    /// if package name is abrark.com and it has user-group with id my-all-readers
    /// so import string will be abrark.com/my-all-readers
    pub groups: Vec<String>,
    pub excluded_groups: Vec<String>,
    pub description: Option<String>,
}

// TODO: Keys should be dynamic
/// This type is needed to deserialize ftd to rust

#[derive(serde::Deserialize, Debug)]
pub struct UserGroupTemp {
    // if package name is abrark.com and it has user-group with id my-all-readers
    // so group string will be abrark.com/my-all-readers
    // keys should be dynamic
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "group")]
    pub groups: Vec<String>,
    #[serde(rename = "-group")]
    pub excluded_group: Vec<String>,
    #[serde(rename = "email")]
    pub email: Vec<String>,
    #[serde(rename = "-email")]
    pub excluded_email: Vec<String>,
    #[serde(rename = "domain")]
    pub domain: Vec<String>,
    #[serde(rename = "-domain")]
    pub excluded_domain: Vec<String>,
    #[serde(rename = "telegram")]
    pub telegram: Vec<String>,
    #[serde(rename = "-telegram")]
    pub excluded_telegram: Vec<String>,
    #[serde(rename = "github")]
    pub github: Vec<String>,
    #[serde(rename = "-github")]
    pub excluded_github: Vec<String>,
    #[serde(rename = "github-team")]
    pub github_team: Vec<String>,
    #[serde(rename = "-github-team")]
    pub excluded_github_team: Vec<String>,
    #[serde(rename = "discord")]
    pub discord: Vec<String>,
    #[serde(rename = "-discord")]
    pub excluded_discord: Vec<String>,
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
    // It will contain all group members, like group, email and -email, etc...
    #[serde(rename = "group-members")]
    group_members: Vec<fpm::library::full_sitemap::KeyValueData>,
}

impl UserGroup {
    pub fn to_group_compat(&self) -> UserGroupCompat {
        let mut group_members = vec![];

        // Group Identities
        group_members.extend(
            self.identities
                .clone()
                .into_iter()
                .map(|i| fpm::library::KeyValueData::from(i.key, i.value)),
        );

        group_members.extend(
            self.excluded_identities.iter().map(|i| {
                fpm::library::KeyValueData::from(format!("-{}", i.key), i.value.to_string())
            }),
        );

        group_members.extend(
            self.groups
                .iter()
                .map(|i| fpm::library::KeyValueData::from("group".to_string(), i.to_string())),
        );

        UserGroupCompat {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            group_members,
        }
    }

    // TODO: Need to handle excluded_identities and excluded_groups
    // Maybe Logic: group.identities + (For all group.groups(g.group - g.excluded_group)).identities
    //              - group.excluded_identities
    pub fn get_identities(&self, config: &fpm::Config) -> fpm::Result<Vec<UserIdentity>> {
        let mut identities = vec![];
        for group in self.groups.iter() {
            dbg!(&group);
            let group =
                fpm::config::user_group_by_id(config, group.as_str())?.ok_or_else(|| {
                    fpm::Error::GroupNotFound {
                        message: format!("group: {}, not found in FPM.ftd", group),
                    }
                })?;
            // Recursive call to get child groups identities
            identities.extend(group.get_identities(config)?)
        }
        identities.extend(self.identities.clone());

        Ok(identities)
    }

    // TODO:
    // This function will check whether given identities are part or given groups or not,
    // It will return true if all are part of provided groups
    // pub fn belongs_to(_identities: &[&str], _groups: &[UserGroup]) -> fpm::Result<bool> {
    //     Ok(false)
    // }
}

impl UserGroupTemp {
    pub fn are_unique(groups: &[UserGroupTemp]) -> Result<bool, String> {
        // TODO: Tell all the repeated ids at once, this will only tell one at a time
        // TODO: todo this we have to count frequencies and return error if any frequency is
        // greater that one
        let mut set = std::collections::HashSet::new();
        for group in groups {
            if set.contains(&group.id) {
                return Err(format!(
                    "user-group ids are not unique: repeated id: {}",
                    group.id
                ));
            }
            set.insert(&group.id);
        }
        Ok(true)
    }

    pub fn user_groups(
        user_groups: Vec<UserGroupTemp>,
    ) -> fpm::Result<std::collections::BTreeMap<String, UserGroup>> {
        Self::are_unique(&user_groups).map_err(|e| {
            crate::sitemap::ParseError::InvalidUserGroup {
                doc_id: "FPM.ftd".to_string(),
                message: e,
                row_content: "".to_string(),
            }
        })?;
        let mut groups = std::collections::BTreeMap::new();
        for group in user_groups.into_iter() {
            groups.insert(group.id.to_string(), group.to_user_group()?);
        }
        Ok(groups)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_user_group(self) -> fpm::Result<UserGroup> {
        use itertools::Itertools;
        let mut identities = vec![];
        let mut excluded_identities = vec![];

        fn to_user_identity(name: &str, values: Vec<String>) -> Vec<UserIdentity> {
            values
                .into_iter()
                .map(|v| UserIdentity::from(name, v.as_str()))
                .collect_vec()
        }

        identities.extend(to_user_identity("email", self.email));
        excluded_identities.extend(to_user_identity("-email", self.excluded_email));
        identities.extend(to_user_identity("domain", self.domain));
        excluded_identities.extend(to_user_identity("-domain", self.excluded_domain));
        identities.extend(to_user_identity("domain", self.telegram));
        excluded_identities.extend(to_user_identity("-telegram", self.excluded_telegram));
        identities.extend(to_user_identity("github", self.github));
        excluded_identities.extend(to_user_identity("-github", self.excluded_github));
        identities.extend(to_user_identity("github-team", self.github_team));
        excluded_identities.extend(to_user_identity("-github-team", self.excluded_github_team));
        identities.extend(to_user_identity("discord", self.discord));
        excluded_identities.extend(to_user_identity("-discord", self.excluded_discord));

        Ok(UserGroup {
            id: self.id,
            description: self.description,
            identities,
            excluded_identities,
            title: self.title,
            groups: self.groups,
            excluded_groups: self.excluded_group,
        })
    }
}

pub fn get_identities(
    config: &crate::Config,
    doc_path: &str,
    is_read: bool,
) -> fpm::Result<Vec<String>> {
    // TODO: cookies or cli parameter

    let readers_writers = if let Some(sitemap) = &config.sitemap {
        if is_read {
            sitemap.readers(doc_path, &config.groups)
        } else {
            sitemap.writers(doc_path, &config.groups)
        }
    } else {
        vec![]
    };

    let identities: fpm::Result<Vec<Vec<UserIdentity>>> = readers_writers
        .into_iter()
        .map(|g| g.get_identities(config))
        .collect();

    let identities = identities?
        .into_iter()
        .flat_map(|x| x.into_iter())
        .map(|identity| identity.to_string())
        .collect();

    Ok(identities)
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
            .map(|(_, g)| g.to_group_compat())
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
            .map(|g| g.to_group_compat())
            .ok_or_else(|| ftd::p1::Error::NotFound {
                key: format!("user-group: `{}` not found", id),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            })?;
        doc.from_json(&g, section)
    }

    pub fn get_identities(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = fpm::library::document::document_full_id(config, doc)?;
        let identities = super::get_identities(config, doc_id.as_str(), true).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: e.to_string(),
                doc_id,
                line_number: section.line_number,
            }
        })?;

        Ok(ftd::Value::List {
            data: identities
                .into_iter()
                .map(|i| ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: i,
                        source: ftd::TextSource::Default,
                    },
                })
                .collect_vec(),
            kind: ftd::p2::Kind::List {
                kind: Box::new(ftd::p2::Kind::String {
                    caption: false,
                    body: false,
                    default: None,
                    is_reference: false,
                }),
                default: None,
                is_reference: false,
            },
        })
    }
}

// fn a_minus_b_ref<'a>(
//     a: &'a [(&'a str, &'a str)],
//     b: &'a [(&'a str, &'a str)],
// ) -> Vec<&'a (&'a str, &'a str)> {
//     let mut excluded: HashMap<_, HashSet<_>> = HashMap::new();
//     for (k, v) in b {
//         if excluded.contains_key(k) {
//             let t = excluded.get_mut(k).unwrap();
//             t.insert(v);
//         } else {
//             let mut t = HashSet::new();
//             t.insert(v);
//             excluded.insert(k, t);
//         }
//     }
//
//     let is_in_b = |k: &str, v: &str| {
//         if let Some(set) = excluded.get(&k) {
//             return set.contains(&v);
//         }
//         false
//     };
//     a.into_iter().filter(|(k, v)| !is_in_b(k, v)).collect_vec()
// }
//
// fn a_minus_b<'a>(
//     a: &'a Vec<(String, String)>,
//     b: &'a Vec<(String, String)>,
// ) -> Vec<(String, String)> {
//     let mut excluded: HashMap<_, HashSet<_>> = HashMap::new();
//     for (k, v) in b {
//         if excluded.contains_key(k) {
//             let t = excluded.get_mut(k).unwrap();
//             t.insert(v);
//         } else {
//             let mut t = HashSet::new();
//             t.insert(v);
//             excluded.insert(k, t);
//         }
//     }
//     let is_in_b = |k: &String, v: &String| {
//         if let Some(set) = excluded.get(&k) {
//             return set.contains(v);
//         }
//         false
//     };
//     //TODO: Remove .map(|(k, v)| (k.to_string(), v.to_string()))
//     a.into_iter()
//         .filter(|(k, v)| !is_in_b(k, v))
//         .map(|(k, v)| (k.to_string(), v.to_string()))
//         .collect_vec()
// }

// pub fn get_group_members(&self, config: &fpm::Config) -> Vec<(String, String)> {
//     if self.import.len() == 0 {
//         return UserGroup::a_minus_b(&self.identities, &self.excluded_identities);
//     }
//
//     let mut group_identities = vec![];
//     for group in self.import.iter() {
//         let (package, group_id) = group
//             .rsplit_once('/')
//             .ok_or_else(|| ftd::p1::Error::ParseError {
//                 message: format!("import_identity: {}, does not contain `/`", group),
//                 doc_id: "FPM.ftd".to_string(),
//                 line_number: 0,
//             })
//             .unwrap();
//
//         // TODO: Remove unwrap
//         let fpm_document = config.get_fpm_document(package).unwrap();
//         let user_groups: Vec<UserGroupTemp> = fpm_document.get("fpm#user-group").unwrap();
//         let user_group = user_groups.into_iter().find(|g| g.id.eq(group_id)).unwrap();
//         let user_group = user_group.to_user_group();
//         let user_group_members = user_group.get_group_members(config);
//         group_identities.extend(user_group_members);
//     }
//
//     //TODO: Remove Clone
//     group_identities.extend(self.identities.clone());
//     return UserGroup::a_minus_b(&group_identities, &self.excluded_identities);
// }
