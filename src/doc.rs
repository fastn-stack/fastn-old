// TODO: make async
pub async fn parse<'a>(
    name: &str,
    source: &str,
    lib: &'a fpm::Library,
) -> ftd::p1::Result<ftd::p2::Document> {
    dbg!(&name, &lib.config.package);
    let current_package = vec![&lib.config.package];
    let mut s = ftd::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib
                    .process(&section, &state.tdoc(&mut Default::default()))
                    .await?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                if module.eq("aia") {
                    s = st.continue_after_import(module.as_str(), None, Some(module.as_str()))?;
                } else {
                    let source =
                        lib.get_with_result(module.as_str(), &st.tdoc(&mut Default::default()))?;
                    s = st.continue_after_import(module.as_str(), Some(source.as_str()), None)?;
                }
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value =
                    resolve_foreign_variable(variable.as_str(), state.document_stack.last());
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}

fn resolve_foreign_variable(variable: &str, document: Option<&ftd::ParsedDocument>) -> ftd::Value {
    dbg!(&variable, &document.unwrap().get_doc_aliases());

    ftd::Value::String {
        text: format!(
            "Time is: {}",
            std::str::from_utf8(
                std::process::Command::new("date")
                    .output()
                    .expect("failed to execute process")
                    .stdout
                    .as_slice()
            )
            .unwrap()
        ),
        source: ftd::TextSource::Header,
    }
}

// No need to make async since this is pure.
pub fn parse_ftd(
    name: &str,
    source: &str,
    lib: &fpm::FPMLibrary,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib.process(&section, &state.tdoc(&mut Default::default()))?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                let source =
                    lib.get_with_result(module.as_str(), &st.tdoc(&mut Default::default()))?;
                s = st.continue_after_import(module.as_str(), Some(source.as_str()), None)?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value =
                    resolve_foreign_variable(variable.as_str(), state.document_stack.last());
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}
