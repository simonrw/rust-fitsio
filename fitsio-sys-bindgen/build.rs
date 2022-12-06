use bindgen::RustTarget;
use pkg_config::Error;
use std::env;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let package_name = "cfitsio >= 3.37";
    let mut config = pkg_config::Config::new();
    config.print_system_libs(true);
    config.print_system_cflags(true);
    match config.probe(package_name) {
        Ok(lib) => {
            let include_args: Vec<_> = lib
                .include_paths
                .into_iter()
                .map(|p| format!("-I{}", p.to_str().unwrap()))
                .collect();
            let bindings = bindgen::builder()
                .header("wrapper.h")
                .block_extern_crate(true)
                .clang_args(include_args)
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
                format!("{package_name} was not found in the pkg-config search path").as_ref(),
            ) {
                let err_msg = format!(
                    "
Cannot find {package_name} on the pkg-config search path.  Consider installing the library for your
system (e.g. through homebrew, apt-get etc.).  Alternatively if it is installed, then add
the directory that contains `cfitsio.pc` on your PKG_CONFIG_PATH, e.g.:

PKG_CONFIG_PATH=<blah> cargo build
"
                );
                std::io::stderr().write_all(err_msg.as_bytes()).unwrap();
                std::process::exit(output.status.code().unwrap());
            }
        }
        Err(e) => panic!("Unhandled error: {:?}", e),
    };
}
