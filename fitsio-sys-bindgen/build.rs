use bindgen::RustTarget;
use pkg_config::Error;
use std::env;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let package_name = "cfitsio";
    match pkg_config::probe_library(package_name) {
        Ok(_) => {
            let bindings = bindgen::builder()
                .header("wrapper.h")
                .block_extern_crate(true)
                .opaque_type("fitsfile")
                .opaque_type("FITSfile")
                .rust_target(RustTarget::Stable_1_0)
                .generate()
                .expect("Unable to generate bindings");
            let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
            bindings
                .write_to_file(out_path.join("bindings.rs"))
                .expect("Couldn't write bindings");
        }
        Err(Error::Failure { output, .. }) => {
            // Handle the case where the user has not installed cfitsio, and thusly it is not on
            // the PKG_CONFIG_PATH
            let stderr = String::from_utf8(output.stderr).unwrap();
            if stderr.contains::<&str>(
                format!(
                    "{} was not found in the pkg-config search path",
                    package_name
                )
                .as_ref(),
            ) {
                let err_msg = format!(
                    "
Cannot find {} on the pkg-config search path.  Consider installing the library for your
system (e.g. through homebrew, apt-get etc.).  Alternatively if it is installed, then add
the directory that contains `cfitsio.pc` on your PKG_CONFIG_PATH, e.g.:

PKG_CONFIG_PATH=<blah> cargo build
",
                    package_name
                );
                std::io::stderr().write_all(err_msg.as_bytes()).unwrap();
                std::process::exit(output.status.code().unwrap());
            }
        }
        Err(e) => panic!("Unhandled error: {:?}", e),
    };
}
