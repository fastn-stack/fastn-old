use crate::library::toc::TocItem;
use crate::library::toc::TocParser;
use crate::library::toc::TocItemCompat;
use crate::library::toc::ParseError;
use crate::library::toc::ToC;
use crate::library::toc::processor;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsingState {
    WaitingForNextItem,
    WaitingForAttributes,
}
#[derive(Debug)]
struct LevelTree {
    level: usize,
    item: TocItem,
}

impl LevelTree {
    fn new(level: usize, item: TocItem) -> Self {
        Self { level, item }
    }
}

fn construct_tree_util(mut elements: Vec<(TocItem, usize)>) -> Vec<TocItem> {
    if elements.is_empty() {
        return vec![];
    }
    let smallest_level = elements.get(0).unwrap().1;
    elements.push((TocItem::default(), smallest_level));
    // println!("Elements: {:#?}", elements);
    let mut tree = construct_tree(elements, smallest_level);
    let _garbage = tree.pop();
    tree.into_iter().map(|x| x.item).collect()
}

fn get_top_level(stack: &[LevelTree]) -> usize {
    stack.last().map(|x| x.level).unwrap()
}

fn construct_tree(elements: Vec<(TocItem, usize)>, smallest_level: usize) -> Vec<LevelTree> {
    let mut stack_tree = vec![];
    let mut num: Vec<u8> = vec![0];
    for (toc_item, level) in elements.into_iter() {
        if level < smallest_level {
            panic!("Level should not be lesser than smallest level");
        }

        if !(stack_tree.is_empty() || get_top_level(&stack_tree) <= level) {
            let top = stack_tree.pop().unwrap();
            let mut top_level = top.level;
            let mut children = vec![top];
            while level < top_level {
                loop {
                    if stack_tree.is_empty() {
                        panic!("Tree should not be empty here")
                    }
                    let mut cur_element = stack_tree.pop().unwrap();
                    if stack_tree.is_empty() || cur_element.level < top_level {
                        // Means found children's parent, needs to append children to its parents
                        // and update top level accordingly
                        // parent level should equal to top_level - 1
                        assert_eq!(cur_element.level as i32, (top_level as i32) - 1);
                        cur_element
                            .item
                            .children
                            .append(&mut children.into_iter().rev().map(|x| x.item).collect());
                        top_level = cur_element.level;
                        children = vec![];
                        stack_tree.push(cur_element);
                        break;
                    } else if cur_element.level == top_level {
                        // if popped element is same as already popped element it is adjacent
                        // element, needs to push into children and find parent in stack
                        children.push(cur_element);
                    } else {
                        panic!(
                            "Stacked elements level should never be greater than top element level"
                        );
                    }
                }
            }
            assert!(level >= top_level);
        }
        let new_toc_item = match &toc_item.is_heading {
            true => {
                // Level reset. Remove all elements > level
                if level < (num.len() - 1) {
                    num = num[0..level + 1].to_vec();
                } else if let Some(i) = num.get_mut(level) {
                    *i = 0;
                }
                toc_item
            }
            false => {
                if level < (num.len() - 1) {
                    // Level reset. Remove all elements > level
                    num = num[0..level + 1].to_vec();
                }
                if let Some(i) = num.get_mut(level) {
                    *i += 1;
                } else {
                    num.insert(level, 1);
                };
                TocItem {
                    number: num.clone(),
                    ..toc_item
                }
            }
        };
        let node = LevelTree::new(level, new_toc_item);

        stack_tree.push(node);
    }
    stack_tree
}

impl TocParser {
    pub fn read_line(&mut self, line: &str) -> Result<(), ParseError> {
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
                        TocItem {
                            title: Some(iter.collect::<String>().trim().to_string()),
                            is_heading: true,
                            ..Default::default()
                        },
                        depth,
                    ));
                    self.state = fpm::library::toc::WaitingForNextItem;
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
            TocItem {
                title: Some(rest.as_str().trim().to_string()),
                ..Default::default()
            },
            depth,
        ));
        self.state = fpm::library::toc::ParsingState::WaitingForAttributes;
        Ok(())
    }

    fn eval_temp_item(&mut self) -> Result<(), ParseError> {
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
                            return Err(ParseError::InvalidTOCItem {
                                doc_id: self.doc_name.clone(),
                                message: "Ambiguous <title>: <URL> evaluation. Multiple colons found. Either specify the complete URL or specify the url as an attribute".to_string(),
                                row_content: current_title.as_str().to_string(),
                            });
                        }
                    }
                };
                TocItem {
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
    fn read_attrs(&mut self, line: &str) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            // Empty line found. Process the temp_item
            self.eval_temp_item()?;
        } else {
            match self.temp_item.clone() {
                Some((i, d)) => match line.split_once(':') {
                    Some(("url", v)) => {
                        self.temp_item = Some((
                            TocItem {
                                url: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("id", v)) => {
                        self.temp_item = Some((
                            TocItem {
                                url: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("font-icon", v)) => {
                        self.temp_item = Some((
                            TocItem {
                                font_icon: Some(v.trim().to_string()),
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("is-disabled", v)) => {
                        self.temp_item = Some((
                            TocItem {
                                is_disabled: v.trim() == "true",
                                ..i
                            },
                            d,
                        ));
                    }
                    Some(("src", v)) => {
                        self.temp_item = Some((
                            TocItem {
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

    fn finalize(self) -> Result<Vec<(TocItem, usize)>, ParseError> {
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
