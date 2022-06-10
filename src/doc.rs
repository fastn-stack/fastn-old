// TODO: make async
pub async fn parse<'a>(
    name: &str,
    source: &str,
    lib: &'a fpm::Library,
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
                let value = lib
                    .process(&section, &state.tdoc(&mut Default::default()))
                    .await?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                if module.eq("aia") {
                    s = st.continue_after_import(module.as_str(), None, Some("aia"))?;
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
        text: "Hence proved, Abrar is Awesome".to_string(),
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
