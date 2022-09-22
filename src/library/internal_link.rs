use ftd::Value::String;
use crate::Error;

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    _config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let toc_items = ToCList::parse(
        section.body(section.line_number, doc.name)?.as_str(),
        doc.name,
    )
        .map_err(|e| ftd::p1::Error::ParseError {
            message: format!("Cannot parse body: {:?}", e),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?
        .items
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<fpm::library::toc::TocItemCompat>>();
    doc.from_json(&toc_items, section)
}

impl ToCList {
    pub fn parse(
        s: &str,
        doc_name: &str,
    ) -> Result<Self, fpm::library::toc::ParseError> {
        let mut parser = TocListParser {
            state: fpm::library::toc::ParsingState::WaitingForNextItem,
            sections: vec![],
            temp_item: None,
            doc_name: doc_name.to_string(),
            file_ids: Default::default(),
        };
        let mut doc="0";
        for (ln,line) in itertools::enumerate(s.split('\n')) {
            doc = s;
        }
        parser.read_doc(doc, doc_name)?;
        if parser.temp_item.is_some() {
            parser.eval_temp_item(doc_name)?;
        }
        Ok(ToCList {
            items: fpm::library::toc::construct_tree_util(parser.finalize()?),
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct ToCList {
    pub items: Vec<fpm::library::toc::TocItem>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TocListParser {
    pub(crate) state: fpm::library::toc::ParsingState,
    pub(crate) sections: Vec<(fpm::library::toc::TocItem, usize)>,
    pub(crate) temp_item: Option<(fpm::library::toc::TocItem, usize)>,
    pub(crate) doc_name: std::string::String,
    pub(crate) file_ids: std::collections::HashMap<std::string::String, std::string::String>,
}

impl TocListParser {
    pub fn read_doc(
        &mut self,
        doc: &str,
        doc_name: &str,
    ) -> Result<(), fpm::library::toc::ParseError> {
        // The row could be one of the 4 things:

        // - Heading
        // - Prefix/suffix item
        // - Separator
        // - ToC item
        if doc.trim().is_empty() {
            return Ok(());
        }

        let mut depth = 0;

        let mut file_ids: std::collections::HashMap <std::string::String, std::string::String> = std::collections::HashMap::new();

        let document_id = fpm::library::convert_to_document_id(doc_name);
        dbg!(&doc_name);

        fn update_id_map(
            file_ids: &mut std::collections::HashMap<std::string::String, std::string::String>,
            line: &str,
            doc_name: &str,
            line_number: usize,
        ) -> fpm::Result<()> {
            println!("inside update id map");
            // returns doc-id from link as String
            fn fetch_doc_id_from_link(link: &str) -> fpm::Result<std::string::String> {
                // link = <document-id>#<slugified-id>
                let doc_id = link.split_once('#').map(|s| s.0);
                match doc_id {
                    Some(id) => Ok(id.to_string()),
                    None => Err(fpm::Error::PackageError {
                        message: format!("Invalid link format {}", link),
                    }),
                }
            }

            let (_header, value) =
                ftd::identifier::segregate_key_value(line, doc_name, line_number)?;
            let document_id = fpm::library::convert_to_document_id(doc_name);

            if let Some(id) = value {
                // check if the current id already exists in the map
                // if it exists then throw error
                if file_ids.contains_key(&id) {
                    return Err(fpm::Error::UsageError {
                        message: format!(
                            "conflicting id: \'{}\' used in doc: \'{}\'",
                            id,
                            document_id
                        ),
                    });
                }

                // mapping id -> <document-id>#<slugified-id>
                let link = format!("{}#{}", document_id, slug::slugify(&id));
                (file_ids.insert(dbg!(id), dbg!(link)));
            }

            Ok(())
        }
        let captured_file_ids: Vec<(std::string::String, usize)> = ftd::p1::parse_file_for_global_ids(doc);
        for (captured_id, ln) in captured_file_ids.iter() {
            println!("inside captured id ");
            dbg!(&captured_id);
            dbg!(&ln);
            let id_map = update_id_map(&mut self.file_ids, captured_id.as_str(), &document_id, *ln);
            dbg!(id_map);
        }

        for (ln, line) in itertools::enumerate(doc.split('\n')) {
            let update_id_map = |file_ids: &mut std::collections::HashMap<std::string::String, std::string::String>, line: &str, doc_name: &str, line_number: usize| -> Result<(), Error> {
                println!("inside update id map");
                // returns doc-id from link as String
                fn fetch_doc_id_from_link(link: &str) -> fpm::Result<std::string::String> {
                    // link = <document-id>#<slugified-id>
                    let doc_id = link.split_once('#').map(|s| s.0);
                    match doc_id {
                        Some(id) => Ok(id.to_string()),
                        None => Err(fpm::Error::PackageError {
                            message: format!("Invalid link format {}", link),
                        }),
                    }
                }

                let (_header, value) =
                    ftd::identifier::segregate_key_value(line, doc_name, ln)?;
                let document_id = fpm::library::convert_to_document_id(doc_name);

                if let Some(id) = value {
                    // check if the current id already exists in the map
                    // if it exists then throw error
                    if file_ids.contains_key(&id) {
                        return Err(fpm::Error::UsageError {
                            message: format!(
                                "conflicting id: \'{}\' used in doc: \'{}\'",
                                id,
                                document_id,
                            )
                        });
                    }

                    // mapping id -> <document-id>#<slugified-id>
                    let link = format!("{}#{}", document_id, slug::slugify(&id));
                    file_ids.insert(dbg!(id), dbg!(link));
                }

                return Ok(());
            };
                let captured_file_ids: Vec<(std::string::String, usize)> = ftd::p1::parse_file_for_global_ids(doc);
                for (captured_id, ln) in captured_file_ids.iter() {
                    println!("inside captured id ");
                    dbg!(&captured_id);
                    dbg!(&ln);
                    let id_map = update_id_map(&mut self.file_ids, captured_id.as_str(), &document_id, *ln);
                    dbg!(id_map);
                }
            dbg!(file_ids);

            let mut iter = line.chars();
            loop {
                match iter.next() {
                    Some(' ') => {
                        depth += 1;
                        iter.next();
                    }
                    Some('-') => {
                        break;
                    }
                    Some('#') => {
                        // Heading can not have any attributes. Append the item and look for the next input
                        self.eval_temp_item(doc_name)?;
                        self.sections.push((
                            fpm::library::toc::TocItem {
                                title: Some(iter.collect::<std::string::String>().to_string()),
                                is_heading: false,
                                ..Default::default()
                            },
                            depth,
                        ));
                        self.state = fpm::library::toc::ParsingState::WaitingForNextItem;
                        return Ok(());
                    }
                    Some(k) => {
                        let l = format!("{}{}", k, iter.collect::<std::string::String>());
                        self.read_id(l.as_str(), doc_name)?;
                        return Ok(());
                        // panic!()
                    }
                    None => {
                        break;
                    }
                }
            }
            let rest: std::string::String = iter.collect();
            self.eval_temp_item(doc_name)?;

            // Stop eager checking, Instead of split and evaluate URL/title, first push
            // The complete string, postprocess if url doesn't exist
            self.temp_item = Some((
                fpm::library::toc::TocItem {
                    title: Some(rest.as_str().trim().to_string()),
                    ..Default::default()
                },
                depth,
            ));
            self.state = fpm::library::toc::ParsingState::WaitingForAttributes;
            return Ok(());
        }
        Ok(())
    }

    pub fn read_id(
        &mut self,
        line: &str,
        doc_name: &str,
    ) -> Result<(), fpm::library::toc::ParseError> {
        let document_id = fpm::library::convert_to_document_id(doc_name);
        if line.trim().is_empty() {
            // Empty line found. Process the temp_item
            self.eval_temp_item(&document_id)?;
        } else {
            match self.temp_item.clone() {
                Some((i, d)) => match line.split_once(':') {
                    Some(("url", v)) => {
                        self.temp_item = Some((
                            fpm::library::toc::TocItem {
                                url: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("id", v)) => {
                        self.temp_item = Some((
                            fpm::library::toc::TocItem {
                                id: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some((k, v)) => {
                        if k.contains("h0")||k.contains("h1")||k.contains("h2")||k.contains("h3")||k.contains("h4")||k.contains("h5")||k.contains("h6")||k.contains("h7"){
                            self.temp_item = Some((
                                fpm::library::toc::TocItem {
                                    title: Some(v.trim().to_string()),
                                    ..i
                                },
                                d,
                            ))
                        };
                    }
                    _ => todo!(),
                },
                _ => panic!("State mismatch"),
            };
        };
        Ok(())
    }

    fn eval_temp_item(&mut self, doc_name: &str) -> Result<(), fpm::library::toc::ParseError> {
        let document_id = fpm::library::convert_to_document_id(doc_name);
        if let Some((toc_item, depth)) = self.temp_item.clone() {
            // Split the line by `:`. title = 0, url = Option<1>
            let resp_item = if toc_item.url.is_none() && toc_item.title.is_some() {
                // URL not defined, Try splitting the title to evaluate the URL
                let current_title = toc_item.title.clone().unwrap();
                let (title, url) = match current_title.as_str().matches(':').count() {
                    1 | 0 => {
                        if let Some((first, second)) = current_title.rsplit_once(':') {
                            let url_id = format!("{}#{}", doc_name, second.trim().to_string());
                            dbg!((Some(first.trim().to_string()), Some(url_id.to_string())))
                        } else {
                            // No matches, i.e. return the current string as title, url as none
                            (Some(current_title), None)
                        }
                    }
                    _ => {
                        // The URL can have its own colons. So match the URL first
                        let url_regex = regex::Regex::new(
                            r#":[ ]?(?P<url>(?:https?)?://(?:[a-zA-Z0-9]+\.)?(?:[A-z0-9]+\.)(?:[A-z0-9]+)(?:[/A-Za-z0-9\?:\&%]+))"#
                        ).unwrap();
                        if let Some(regex_match) = url_regex.find(current_title.as_str()) {
                            let curr_title = current_title.as_str();
                            (
                                Some(curr_title[..regex_match.start()].trim().to_string()),
                                Some(
                                    curr_title[regex_match.start()..regex_match.end()]
                                        .trim_start_matches(':')
                                        .trim()
                                        .to_string(),
                                ),
                            )
                        } else {
                            return Err(fpm::library::toc::ParseError::InvalidTOCItem {
                                doc_id: self.doc_name.clone(),
                                message: "Ambiguous <title>: <URL> evaluation. Multiple colons found. Either specify the complete URL or specify the url as an attribute".to_string(),
                                row_content: current_title.as_str().to_string(),
                                line_number: 0
                            });
                        }
                    }
                };
                fpm::library::toc::TocItem {
                    title,
                    url,
                    ..toc_item
                }
            } else {
                toc_item
            };
            self.sections.push((resp_item, depth))
        }
        self.temp_item = None;
        Ok(())
    }
    pub fn finalize(
        self,
    ) -> Result<Vec<(fpm::library::toc::TocItem, usize)>, fpm::library::toc::ParseError> {
        Ok(self.sections)
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    macro_rules! p {
        ($s:expr, $t: expr,) => {
            p!($s, $t)
        };
        ($s:expr, $t: expr) => {
            assert_eq!(
                super::ToCList::parse($s, "test_doc").unwrap_or_else(|e| panic!("{}", e)),
                $t
            )
        };
    }

    #[test]
    fn parse() {
        p!(
            &indoc!(
                "
        -- h0: Title1
        id: t1

        -- h1: Title2
        id: t2


        -- ftd.column h0:
        caption title:
        open: true
        append-at: foo

        --- ftd.text: $title

        --- ftd.column:
        id: foo



        -- ftd.column h1:
        caption title:
        open: true
        append-at: foo

        --- ftd.text: $title

        --- ftd.column:
        id: foo

        "
            ),
            super::ToCList {
                items: vec![fpm::library::toc::TocItem {
                    title: Some(format!("- h0")),
                    id: Some(format!("t1")),
                    url: Some("/test_doc/#Title1".to_string()),
                    number: vec![1],
                    is_heading: false,
                    is_disabled: false,
                    img_src: None,
                    font_icon: None,
                    path: None,
                    children: vec![],
                    },
                    fpm::library::toc::TocItem {
                    title: Some(format!("- h1")),
                    id: Some(format!("t2")),
                    url: Some("/test_doc/#Title2".to_string()),
                    is_heading: false,
                    number: vec![2],
                    is_disabled: false,
                    img_src: None,
                    font_icon: None,
                    children: vec![],
                    path: None
                    }
                ]
            },
        );
    }

    #[test]
    fn parse_heading() {
        p!(
            &indoc!(
                "
        -- h0: Title1
        id: t1
        "
            ),
            super::ToCList {
                items: vec![fpm::library::toc::TocItem {
                    title: Some(format!("- h0")),
                    id: Some(format!("t1")),
                    url: Some("/test_doc/#Title1".to_string()),
                    number: vec![1],
                    is_disabled: false,
                    is_heading: false,
                    img_src: None,
                    font_icon: None,
                    children: vec![],
                    path: None
                }]
            }
        );
    }

    #[test]
    fn parse_simple_with_num() {
        p!(
            &indoc!(
                "
        -- h0: Title1
        id: t1

        -- h0: Title2
        id: t2
        "
            ),
            super::ToCList {
                items: vec![
                    fpm::library::toc::TocItem {
                        title: Some(format!("- h0")),
                        id: Some(format!("t1")),
                        url: Some("/test_doc/#Title1".to_string()),
                        is_heading: false,
                        number: vec![1],
                        is_disabled: false,
                        img_src: None,
                        font_icon: None,
                        children: vec![],
                        path: None
                    },
                    fpm::library::toc::TocItem {
                        title: Some(format!("- h0")),
                        id: Some(format!("t2")),
                        url: Some("/test_doc/#Title2".to_string()),
                        is_heading: false,
                        number: vec![2],
                        is_disabled: false,
                        img_src: None,
                        font_icon: None,
                        children: vec![],
                        path: None
                    }
                ]
            }
        );
    }
}
