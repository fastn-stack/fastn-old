pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    _config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    doc.from_json(&vec![1], &kind, 1)
}
