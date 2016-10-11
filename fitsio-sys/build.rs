extern crate pkg_config;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    pkg_config::probe_library("cfitsio").unwrap();
}
