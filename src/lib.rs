extern crate self as fpm;

mod build;
mod check;
mod config;
mod document;
mod library;
mod sync;
mod utils;

pub use build::build;
pub use check::check;
pub use config::Package;
pub(crate) use document::{process_dir, Document};
pub use library::Library;
pub use sync::sync;
pub(crate) use utils::get_timestamp_nanosecond;

pub fn fpm_ftd() -> &'static str {
    include_str!("../fpm.ftd")
}

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
