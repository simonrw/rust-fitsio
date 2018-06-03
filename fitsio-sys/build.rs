extern crate pkg_config;

use pkg_config::Error;
use std::io::Write;

fn main() {
    let package_name = "cfitsio";
    match pkg_config::probe_library(package_name) {
        Ok(_) => {}
        Err(Error::Failure { output, .. }) => {
            // Handle the case where the user has not installed cfitsio, and thusly it is not on
            // the PKG_CONFIG_PATH
            let stderr = String::from_utf8(output.stderr).unwrap();
            if stderr.contains::<&str>(
                format!(
                    "{} was not found in the pkg-config search path",
                    package_name
                ).as_ref(),
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
                std::io::stderr().write(err_msg.as_bytes()).unwrap();
                std::process::exit(output.status.code().unwrap());
            }
        }
        Err(e) => panic!("Unhandled error: {:?}", e),
    };
}
