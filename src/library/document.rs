/*
document filename
foo/abc.ftd

document id
/foo/abc/
/foo/abc/-/x/y/ --> full id
*/

pub mod processor {

    pub fn document_id<'a>(
        _section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let document_id = doc_id
            .split_once("/-/")
            .map(|x| x.0)
            .unwrap_or_else(|| &doc_id)
            .trim_matches('/');

        Ok(ftd::Value::String {
            text: format!("/{}/", document_id),
            source: ftd::TextSource::Default,
        })
    }
    pub fn document_full_id<'a>(
        _section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let full_document_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        Ok(ftd::Value::String {
            text: format!("/{}/", full_document_id.trim_matches('/')),
            source: ftd::TextSource::Default,
        })
    }
    pub fn document_filename<'a>(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        unimplemented!()
    }
    pub fn document_suffix<'a>(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        unimplemented!()
    }
}
