extern crate self as fpm;

mod build;
mod check;
mod config;
mod library;
mod sync;
mod utils;

pub use build::build;
pub use check::check;
pub use config::Package;
pub use library::Library;
pub use sync::sync;
pub use utils::process_dir;

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
