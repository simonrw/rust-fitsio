#[cfg(not(feature = "fitsio-src"))]
fn bind_cfitsio() {
    use pkg_config::Error;
    use std::io::Write;

    let package_name = "cfitsio >= 3.47";
    let mut config = pkg_config::Config::new();
    config.print_system_libs(true);
    config.print_system_cflags(true);
    match config.probe(package_name) {
        Ok(_) => {}
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

#[cfg(feature = "fitsio-src")]
fn bind_cfitsio() {
    use std::env::var;
    use std::path::PathBuf;

    let cfitsio_project_dir = PathBuf::from("ext/cfitsio");
    if !cfitsio_project_dir.exists() {
        panic!(
            "Expected to find cfitsio source directory {}",
            cfitsio_project_dir.display()
        );
    }
    // Make sure the source directory isn't empty.
    match std::fs::read_dir(&cfitsio_project_dir) {
        Ok(mut d) => {
            if let None = d.next() {
                panic!("cfitsio source directory ext/cfitsio is empty!");
            }
        }
        _ => panic!("Could not read from cfitsio source directory ext/cfitsio !"),
    }

    // Translate rustc optimisation levels to things a C compiler can
    // understand. I don't know if all C compilers agree here, but it should
    // at least work for gcc.
    let opt_level: String = match var("OPT_LEVEL").as_ref().map(|o| o.as_str()) {
        Err(_) => panic!("Something wrong with OPT_LEVEL"),
        // gcc doesn't handle 'z'. Just set it to 's', which also optimises
        // for size.
        Ok("z") => "s",
        Ok(o) => o,
    }
    .to_string();

    // Run the contigure script. I'd use the autotools crate here, but it
    // always outputs two of --{enable,disable}-{shared,static}, none of
    // which is supported by the cfitsio configure script! So, just run the
    // script manually.
    let dst = PathBuf::from(var("OUT_DIR").unwrap());

    if cfitsio_project_dir.join("Makefile").is_file() {
        std::process::Command::new("make")
            .arg("clean")
            .current_dir(&cfitsio_project_dir)
            .spawn()
            .expect("Couldn't run cfitsio make clean")
            .wait()
            .expect("Failed to wait on child");
    }

    std::process::Command::new("sh")
        .args(&[
            "./configure",
            &format!("--prefix={}", dst.display()),
            // cfitsio should always be built with reentrant support.
            "--enable-reentrant",
            // curl functionality is not used through rust-fitsio.
            "--disable-curl",
        ])
        .env("CFLAGS", &format!("-Wall -O{} -fPIE", opt_level))
        .current_dir(&cfitsio_project_dir)
        .spawn()
        .expect("Couldn't run cfitsio configure script")
        .wait()
        .expect("Failed to wait on child");

    std::process::Command::new("make")
        .arg("-j")
        .arg("install")
        .current_dir(&cfitsio_project_dir)
        .spawn()
        .expect("Couldn't run cfitsio makefile")
        .wait()
        .expect("Failed to wait on child");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=cfitsio");
}

fn main() {
    bind_cfitsio();
}
