use crate::utils::HasElements;
use std::iter::FromIterator;
//
// fn render(
//     file: &str,
//     config: &fpm::Config,
//     base_url: &str,
//     ignore_failed: bool,
// ) -> fpm::Result<()> {
//     let dependencies = if let Some(package) = config.package.translation_of.as_ref() {
//         let mut deps = package
//             .get_flattened_dependencies()
//             .into_iter()
//             .unique_by(|dep| dep.package.name.clone())
//             .collect_vec();
//         deps.extend(
//             config
//                 .package
//                 .get_flattened_dependencies()
//                 .into_iter()
//                 .unique_by(|dep| dep.package.name.clone()),
//         );
//         deps
//     } else {
//         config
//             .package
//             .get_flattened_dependencies()
//             .into_iter()
//             .unique_by(|dep| dep.package.name.clone())
//             .collect_vec()
//     };
//     let mut asset_documents = std::collections::HashMap::new();
//     asset_documents.insert(
//         config.package.name.clone(),
//         config.package.get_assets_doc(config, base_url).await?,
//     );
//     for dep in &dependencies {
//         asset_documents.insert(
//             dep.package.name.clone(),
//             dep.package.get_assets_doc(config, base_url).await?,
//         );
//     }
//
//     match (
//         config.package.translation_of.as_ref(),
//         config.package.translations.has_elements(),
//     ) {
//         (Some(_), true) => {
//             // No package can be both a translation of something and has its own
//             // translations, when building `config` we ensured this was rejected
//             unreachable!()
//         }
//         (Some(original), false) => {
//             build_with_original(
//                 config,
//                 original,
//                 file,
//                 base_url,
//                 ignore_failed,
//                 &asset_documents,
//             )
//             .await
//         }
//         (None, false) => {
//             build_simple(config, file, base_url, ignore_failed, &asset_documents).await
//         }
//         (None, true) => {
//             build_with_translations(config, file, base_url, ignore_failed, &asset_documents).await
//         }
//     }?;
//
//     for dep in dependencies {
//         let static_files = std::collections::BTreeMap::from_iter(
//             fpm::get_documents(config, &dep.package)
//                 .await?
//                 .into_iter()
//                 .filter(|file_instance| {
//                     matches!(file_instance, fpm::File::Static(_))
//                         || matches!(file_instance, fpm::File::Code(_))
//                         || matches!(file_instance, fpm::File::Image(_))
//                 })
//                 .collect::<Vec<fpm::File>>()
//                 .into_iter()
//                 .map(|v| (v.get_id(), v)),
//         );
//         process_files(
//             config,
//             &dep.package,
//             &static_files,
//             file,
//             base_url,
//             ignore_failed,
//             &asset_documents,
//         )
//         .await?;
//     }
//
//     Ok(())
// }
