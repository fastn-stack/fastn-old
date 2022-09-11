pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    _config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let toc_items = fpm::library::toc::Toc::parse(
        section.body(section.line_number, doc.name)?.as_str(),
        doc.name,
    )
        .map_err(|e| ftd::p1::Error::fpm::library::toc::ParseError {
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

impl TocListParser {
    pub fn read_toc(&mut self, line: &str) -> Result<(), fpm::library::toc::ParseError> {
        // The row could be one of the 4 things:

        // - Heading
        // - Prefix/suffix item
        // - Separator
        // - ToC item
        if line.trim().is_empty() {
            return Ok(());
        }
        let mut iter = line.chars();
        let mut depth = 0;
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
                    dbg!(self.eval_temp_item())?;
                    self.sections.push((
                        fpm::library::toc::TocItem {
                            title: Some(iter.collect::<String>().trim().to_string()),
                            is_heading: true,
                            ..Default::default()
                        },
                        depth,
                    ));
                    self.state = fpm::library::toc::ParsingState::WaitingForNextItem;
                    return Ok(());
                }
                Some(k) => {
                    let l = format!("{}{}", k, iter.collect::<String>());
                    self.read_attrs(l.as_str())?;
                    return Ok(());
                    // panic!()
                }
                None => {
                    break;
                }
            }
        }
        let rest: String = iter.collect();
        self.eval_temp_item()?;

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
        Ok(())
    }

    fn eval_temp_item(&mut self) -> Result<(), fpm::library::toc::ParseError> {
        if let Some((toc_item, depth)) = self.temp_item.clone() {
            // Split the line by `:`. title = 0, url = Option<1>
            let resp_item = if toc_item.url.is_none() && toc_item.title.is_some() {
                // URL not defined, Try splitting the title to evaluate the URL
                let current_title = toc_item.title.clone().unwrap();
                let (title, url) = match current_title.as_str().matches(':').count() {
                    1 | 0 => {
                        if let Some((first, second)) = current_title.rsplit_once(':') {
                            (
                                Some(first.trim().to_string()),
                                Some(second.trim().to_string()),
                            )
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
    fn read_attrs(&mut self, line: &str) -> Result<(), fpm::library::toc::ParseError> {
        if line.trim().is_empty() {
            // Empty line found. Process the temp_item
            self.eval_temp_item()?;
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
                                url: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("font-icon", v)) => {
                        self.temp_item = Some((
                            fpm::library::toc::TocItem {
                                font_icon: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("is-disabled", v)) => {
                        self.temp_item = Some((
                            fpm::library::toc::TocItem {
                                is_disabled: v.trim() == "true",
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("src", v)) => {
                        self.temp_item = Some((
                            fpm::library::toc::TocItem {
                                img_src: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    _ => todo!(),
                },
                _ => panic!("State mismatch"),
            };
        };
        Ok(())
    }

    fn finalize(self) -> Result<Vec<(fpm::library::toc::TocItem, usize)>, fpm::library::toc::ParseError> {
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
                super::ToC::parse($s, "test_doc").unwrap_or_else(|e| panic!("{}", e)),
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
            super::ToC {
                items: vec![
                    super::TocItem {
                        title: Some(format!("Title1")),
                        id: t1,
                        url: Some("Title1".to_string()),
                        number: vec![],
                        is_heading: true,
                        children: vec![
                            super::TocItem {
                                title: Some(format!("Title2")),
                                id: t2,
                                url: Some("Title2".to_string()),
                                number: vec![1,1],
                                is_heading: true,
                                children: vec![],
                            }
                        ],
                    }
                ]
            }
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
            super::ToC {
                items: vec![super::TocItem {
                    title: Some(format!("Title1")),
                    id: t1,
                    url: Some("Title1".to_string()),
                    number: vec![],
                    is_heading: true,
                    children: vec![],
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
            super::ToC {
                items: vec![
                    super::TocItem {
                        title: Some(format!("Title1")),
                        is_heading: true,
                        id: t1,
                        url: Some("Title1".to_string()),
                        number: vec![1],
                        children: vec![],
                    },
                    super::TocItem {
                        title: Some(format!("Title2")),
                        is_heading: true,
                        id: t2,
                        url: Some("Title2".to_string()),
                        number: vec![1],
                        children: vec![],
                    }
                ]
            }
        );
    }
}
