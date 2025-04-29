use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

fn generate_bindings<'p>(include_paths: impl Iterator<Item = &'p PathBuf>) {
    #[cfg(feature = "with-bindgen")]
    {
        let out_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

        bindgen::builder()
            .header("wrapper.h")
            .block_extern_crate(true)
            .clang_args(include_paths.map(|p| format!("-I{}", p.to_str().unwrap())))
            .opaque_type("fitsfile")
            .opaque_type("FITSfile")
            .rust_target(bindgen::RustTarget::stable(47, 0).unwrap_or_else(|_| unreachable!()))
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings");
    }

    #[cfg(not(feature = "with-bindgen"))]
    {
        let _ = include_paths;
    }
}

#[cfg(feature = "fitsio-src")]
fn main() {
    use cmake::Config;

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
            if d.next().is_none() {
                panic!("cfitsio source directory ext/cfitsio is empty!");
            }
        }
        _ => panic!("Could not read from cfitsio source directory ext/cfitsio !"),
    }

    generate_aliases_mod_file(std::iter::once(&cfitsio_project_dir));

    // Translate rustc optimisation levels to things a C compiler can
    // understand. I don't know if all C compilers agree here, but it should
    // at least work for gcc.
    let opt_level = match std::env::var("OPT_LEVEL").as_ref().map(|o| o.as_str()) {
        Err(_) => panic!("Something wrong with OPT_LEVEL"),
        // gcc doesn't handle 'z'. Just set it to 's', which also optimises
        // for size.
        Ok("z") => "s",
        Ok(o) => o,
    }
    .to_string();

    let opt_flag = format!("-O{opt_level}");

    let dst = Config::new("ext/cfitsio")
        .define("UseCurl", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("USE_PTHREADS", "ON")
        .cflag(opt_flag)
        .cflag("-fPIE")
        .build();

    generate_bindings(std::iter::once(&dst.join("include")));

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=cfitsio");
}

#[cfg(not(feature = "fitsio-src"))]
fn main() {
    // `msys2` does not report the version of cfitsio correctly, so ignore the version specifier for now.
    let package_name = if cfg!(windows) {
        let msg = "No version specifier available for pkg-config on windows, so the version of cfitsio used when compiling this program is unspecified";
        println!("cargo:warning={msg}");
        "cfitsio"
    } else {
        "cfitsio >= 3.37"
    };
    let mut config = pkg_config::Config::new();
    config.print_system_libs(true);
    config.print_system_cflags(true);
    match config.probe(package_name) {
        Ok(lib) => {
            generate_bindings(lib.include_paths.iter());
            generate_aliases_mod_file(lib.include_paths.iter());
        }
        Err(e) => {
            if let pkg_config::Error::Failure { output, .. } = &e {
                // Handle the case where the user has not installed cfitsio, and thusly it is not on
                // the PKG_CONFIG_PATH
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains(
                    format!("{package_name} was not found in the pkg-config search path").as_str(),
                ) {
                    eprintln!(
                        "
    Cannot find {package_name} on the pkg-config search path.  Consider installing the library for your
    system (e.g. through homebrew, apt-get etc.).  Alternatively if it is installed, then add
    the directory that contains `cfitsio.pc` on your PKG_CONFIG_PATH, e.g.:

    PKG_CONFIG_PATH=<blah> cargo build
    "
                    );
                    std::process::exit(output.status.code().unwrap_or(1));
                }
            }
            panic!("Unhandled error: {:?}", e);
        }
    };
}

fn generate_aliases_mod_file<'p>(include_paths: impl Iterator<Item = &'p PathBuf>) {
    let out_dir = env::var("OUT_DIR").expect("set by cargo");

    let mut long_name_header = PathBuf::new();
    let mut long_name_header_found = false;
    for include_path in include_paths {
        long_name_header = include_path.join("longnam.h");
        if long_name_header.exists() {
            long_name_header_found = true;
            break;
        } else {
            long_name_header.clear();
        };
    }

    let out = PathBuf::from(out_dir).join("aliases.rs");
    if long_name_header_found {
        // We've found the fits long names that this library was compiled with,
        // now let's alias them.
        let mut file = BufReader::new(match File::open(&long_name_header) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("There was a problem attempting to read {long_name_header:?}");
                panic!("{}", e);
            }
        });
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).expect("file can be read");

        #[cfg(not(feature = "bindgen"))]
        let mut buffer2 = String::new();

        let mut aliases = Vec::new();
        // fits_open_file is special and has a dirty macro associated with it:
        // #define fits_open_file(A, B, C, D)  ffopentest( CFITSIO_SONAME, A, B, C, D)
        // Include this alias manually.
        aliases.push(("fits_open_file", "ffopen"));

        // These are the other functions to handle carefully.
        let bad_long_names = [
            "fits_open_file",        // Handled above
            "fits_parse_output_url", // Not included in this crate?
        ];

        // There may be functions missing in the crate's provided bindings. Find
        // them and don't allow long names to be provided for them.
        #[cfg(not(feature = "bindgen"))]
        let mut available_short_names = Vec::new();
        #[cfg(not(feature = "bindgen"))]
        {
            #[cfg(target_pointer_width = "64")]
            let filename = "src/bindings_64.rs";
            #[cfg(target_pointer_width = "32")]
            let filename = "src/bindings_32.rs";

            let mut file = BufReader::new(match File::open(filename) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("There was a problem attempting to read {filename:?}");
                    panic!("{}", e);
                }
            });
            file.read_to_string(&mut buffer2).expect("file can be read");
            for line in buffer2.lines() {
                if line.trim_ascii_start().starts_with("pub fn ff") {
                    if let Some(fn_name) = line
                        .split_ascii_whitespace()
                        .nth(2)
                        .and_then(|fn_name| fn_name.strip_suffix('('))
                    {
                        available_short_names.push(fn_name);
                    }
                }
            }
        }

        'line: for line in buffer.lines() {
            if line.starts_with("#define") && line.contains("fits_") {
                let mut macro_define_elems = line.split_ascii_whitespace().skip(1);
                if let (Some(long_name), Some(fitsio_name)) =
                    (macro_define_elems.next(), macro_define_elems.next())
                {
                    // Handle any last trickery.
                    if macro_define_elems.count() != 0 {
                        continue;
                    }
                    for bad_long_name in bad_long_names {
                        if long_name.contains(bad_long_name) {
                            continue 'line;
                        }
                    }
                    #[cfg(not(feature = "bindgen"))]
                    if !available_short_names.contains(&fitsio_name) {
                        continue;
                    }

                    aliases.push((long_name, fitsio_name));
                }
            }
        }

        // Now write out these aliases.
        let mut out_file = BufWriter::new(match File::create(&out) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("There was a problem attempting to create a file at {out:?}");
                panic!("{}", e);
            }
        });
        for (long, short) in aliases {
            writeln!(&mut out_file, "pub use crate::{short} as {long};")
                .expect("file can be written");
        }
    } else {
        // The long names include file couldn't be found. Use the pre-filled
        // default aliases file instead.
        // N.B. fitsio.h includes longnam.h, so the following code probably
        // never runs.
        match std::fs::copy("default-aliases.rs", &out) {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "There was a problem attempting to copy from 'default-aliases.rs' to {out:?}"
                );
                panic!("{}", e);
            }
        }
    }
}
