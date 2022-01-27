pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    // doc.from_json(&toc_items, section)
    todo!();
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct TocItem {
    pub id: Option<String>,
    pub title: ftd::Rendered,
    pub url: Option<String>,
    pub number: Vec<u8>,
    pub is_disabled: bool,
    pub img_src: Option<String>,
    pub font_icon: Option<String>,
    pub children: Vec<TocItem>,
}

#[derive(Debug, Clone, PartialEq)]
enum ParsingState {
    WaitingForNextItem,
    WaitingForAttributes,
    // WaitingForTitle((String, usize)),
}

pub struct TocParser {
    state: ParsingState,
    sections: Vec<(TocItem, usize)>,
    temp_item: Option<(TocItem, usize)>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("InputError: {0}")]
    InputError(String),
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
    for (toc_item, level) in elements.into_iter() {
        if level < smallest_level {
            panic!("Level should not be lesser than smallest level");
        }
        let node = LevelTree::new(level, toc_item);
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
                        //println!(
                        //    "TopLevel: {}, CurrentLevel: {}",
                        //    top_level, cur_element.level
                        //);

                        // println!("Children: {:?}", children);
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
        dbg!(&line, &self.sections);
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
                    self.sections.push((
                        TocItem {
                            title: ftd::markdown_line(iter.collect::<String>().trim()),
                            ..Default::default()
                        },
                        depth,
                    ));
                    self.state = ParsingState::WaitingForNextItem;
                    return Ok(());
                }
                Some(k) => {
                    let l = format!("{}{}", k, iter.collect::<String>());
                    self.read_attrs(l.as_str());
                    return Ok(());
                    // panic!()
                }
                None => {
                    break;
                }
            }
        }
        let rest: String = iter.collect();
        self.eval_temp_item();
        // Split the line by `:`. title = 0, url = Option<1>
        let (t, u) = match rest.rsplit_once(":") {
            Some((i, v)) => (i.trim().to_string(), Some(v.trim().to_string())),
            None => (rest.trim().to_string(), None),
        };
        self.temp_item = Some((
            TocItem {
                title: ftd::markdown_line(t.as_str()),
                url: u,
                ..Default::default()
            },
            depth,
        ));
        self.state = ParsingState::WaitingForAttributes;
        Ok(())
    }

    fn eval_temp_item(&mut self) -> Result<(), ParseError> {
        match self.temp_item.clone() {
            Some((t, u)) => self.sections.push((t, u)),
            _ => (),
        };
        self.temp_item = None;
        Ok(())
    }
    fn read_attrs(&mut self, line: &str) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            // Empty line found. Process the temp_item
            self.eval_temp_item();
        } else {
            match self.temp_item.clone() {
                Some((i, d)) => match line.rsplit_once(":") {
                    Some(("url", v)) => {
                        self.temp_item = Some((
                            TocItem {
                                url: Some(v.trim().to_string()),
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
        // if self.state != ParsingState::WaitingF {
        //     return Err(ParseError::InputError("title not found".to_string()));
        // };
        Ok(self.sections)
    }
}

impl ToC {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let mut parser = TocParser {
            state: ParsingState::WaitingForNextItem,
            sections: vec![],
            temp_item: None,
        };
        for line in s.split('\n') {
            let state = parser.state.clone();
            parser.read_line(line)?;
        }
        if parser.temp_item.is_some() {
            parser.eval_temp_item();
        }
        Ok(ToC {
            items: construct_tree_util(parser.finalize()?),
        })
    }
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct ToC {
    pub items: Vec<TocItem>,
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
                super::ToC::parse($s).unwrap_or_else(|e| panic!("{}", e)),
                $t
            )
        };
    }

    #[test]
    fn parse() {
        p!(
            &indoc!(
                "
        # Hello World!

        - Test Page: /test-page/

        # Title One

        - Home Page
          url: /home/
          # Nested Title
          - Nested Link
            url: /home/nested-link/
          - Nested Link Two: /home/nested-link-two/
            - Further Nesting: /home/nested-link-two/further-nested/
        "
            ),
            super::ToC {
                items: vec![
                    super::TocItem {
                        title: ftd::markdown_line("Hello World!"),
                        id: None,
                        url: None,
                        number: vec![1],
                        is_disabled: false,
                        img_src: None,
                        font_icon: None,
                        children: vec![]
                    },
                    super::TocItem {
                        title: ftd::markdown_line("Test Page"),
                        id: None,
                        url: Some("/test-page/".to_string()),
                        number: vec![1],
                        is_disabled: false,
                        img_src: None,
                        font_icon: None,
                        children: vec![]
                    },
                    super::TocItem {
                        title: ftd::markdown_line("Home Page"),
                        id: None,
                        url: Some("/home/".to_string()),
                        number: vec![1],
                        is_disabled: false,
                        img_src: None,
                        font_icon: None,
                        children: vec![
                            super::TocItem {
                                id: None,
                                title: ftd::markdown_line("Nested Link"),
                                url: Some("/home/nested-link/".to_string(),),
                                number: vec![],
                                is_disabled: false,
                                img_src: None,
                                font_icon: None,
                                children: vec![],
                            },
                            super::TocItem {
                                id: None,
                                title: ftd::markdown_line("Nested Link Two"),
                                url: Some("/home/nested-link-two/".to_string()),
                                number: vec![],
                                is_disabled: false,
                                img_src: None,
                                font_icon: None,
                                children: vec![super::TocItem {
                                    id: None,
                                    title: ftd::markdown_line("Further Nesting"),
                                    url: Some("/home/nested-link-two/further-nested/".to_string()),
                                    number: vec![],
                                    is_disabled: false,
                                    img_src: None,
                                    font_icon: None,
                                    children: vec![],
                                },],
                            },
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
        # Home Page
        "
            ),
            super::ToC {
                items: vec![super::TocItem {
                    title: ftd::markdown_line("Home Page"),
                    id: None,
                    url: None,
                    number: vec![],
                    is_disabled: false,
                    img_src: None,
                    font_icon: None,
                    children: vec![]
                }]
            }
        );
    }

    #[test]
    fn parse_simple_with_num() {
        p!(
            &indoc!(
                "
        - Home Page: /home-page/
          numbering: 
        "
            ),
            super::ToC {
                items: vec![super::TocItem {
                    title: ftd::markdown_line("Home Page"),
                    id: None,
                    url: Some("/home-page/".to_string()),
                    number: vec![],
                    is_disabled: false,
                    img_src: None,
                    font_icon: None,
                    children: vec![]
                }]
            }
        );
    }
}
