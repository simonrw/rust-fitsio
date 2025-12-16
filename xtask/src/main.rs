use clap::{Parser, ValueEnum};
use std::process::{Command, ExitCode};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Build system tasks for rust-fitsio")]
enum Args {
    /// Run tests
    Test {
        /// Rust version to use (e.g., "stable", "nightly")
        #[arg(short = 'r', long, default_value = "stable")]
        rust_version: String,

        /// Which test to run
        #[arg(short = 't', long, default_value = "all")]
        test: TestType,

        /// Extra flags to pass to clippy command
        #[arg(long, allow_hyphen_values = true, default_value = "")]
        extra_clippy_flags: String,

        /// Continue with tests after failure
        #[arg(long, default_value_t = false)]
        no_fail_fast: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestType {
    Workspace,
    Clippy,
    FullExample,
    Array,
    FitsioSrc,
    FitsioSrcAndCmake,
    FitsioSrcAndCmakeAndBindgen,
    FitsioSrcAndBindgen,
    Bindgen,
    All,
}

struct TestRunner {
    rust_version: String,
    no_fail_fast: bool,
}

impl TestRunner {
    fn new(rust_version: String, no_fail_fast: bool) -> Self {
        Self {
            rust_version,
            no_fail_fast,
        }
    }

    fn print_preamble(&self) {
        println!("Rust version:");
        let _ = Command::new("rustc")
            .arg(format!("+{}", self.rust_version))
            .arg("--version")
            .status();
        println!("Cargo version:");
        let _ = Command::new("cargo")
            .arg(format!("+{}", self.rust_version))
            .arg("--version")
            .status();
        println!();
    }

    fn run_cargo(&self, args: &[&str]) -> bool {
        let mut cmd = Command::new("cargo");
        cmd.arg(format!("+{}", self.rust_version));
        cmd.args(args);

        println!("Running {:?}", cmd);
        match cmd.status() {
            Ok(status) => {
                println!();
                if !status.success() {
                    eprintln!("test failed with exit code {}", status.code().unwrap_or(-1));
                    if !self.no_fail_fast {
                        std::process::exit(status.code().unwrap_or(1));
                    }
                    return false;
                }
                true
            }
            Err(e) => {
                eprintln!("Failed to run command: {}", e);
                if !self.no_fail_fast {
                    std::process::exit(1);
                }
                false
            }
        }
    }

    fn print_cfitsio_version_with_features(&self, features: &[&str], default_features: bool) {
        let mut cmd = Command::new("cargo");
        cmd.args([
            "run",
            "--package",
            "fitsio-sys",
            "--example",
            "print_version",
        ]);
        for feature in features {
            cmd.args(["--features", feature]);
        }
        if !default_features {
            cmd.arg("--no-default-features");
        }
        println!("Running {:?}", cmd);
        let _ = cmd.status();
    }

    fn run_test_workspace(&self) {
        self.print_cfitsio_version_with_features(&[], true);
        self.run_cargo(&["nextest", "run"]);
    }

    fn run_test_clippy(&self, extra_clippy_flags: &str) {
        let mut args = vec![
            "clippy",
            "--",
            "-D",
            "warnings",
            "-A",
            "clippy::non-send-fields-in-send-ty",
        ];

        let split_args =
            shlex::split(extra_clippy_flags).expect("invalid extra clippy args format");
        args.extend(split_args.iter().map(AsRef::<str>::as_ref));
        self.run_cargo(&args);
    }

    fn run_test_full_example(&self) {
        self.run_cargo(&[
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--example",
            "full_example",
        ]);
    }

    fn run_test_array(&self) {
        self.run_cargo(&[
            "nextest",
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--features",
            "array",
        ]);
    }

    fn run_test_fitsio_src(&self) {
        self.print_cfitsio_version_with_features(&["fitsio-src"], true);
        self.run_cargo(&[
            "nextest",
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--features",
            "fitsio-src",
        ]);
    }

    fn run_test_fitsio_src_and_cmake(&self) {
        self.print_cfitsio_version_with_features(&["fitsio-src", "src-cmake"], true);
        self.run_cargo(&[
            "nextest",
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--features",
            "fitsio-src",
            "--features",
            "src-cmake",
        ]);
    }

    fn run_test_fitsio_src_and_cmake_and_bindgen(&self) {
        self.print_cfitsio_version_with_features(
            &["fitsio-src", "src-cmake", "with-bindgen"],
            true,
        );
        self.run_cargo(&[
            "nextest",
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--features",
            "fitsio-src",
            "--features",
            "src-cmake",
            "--features",
            "bindgen",
        ]);
    }

    fn run_test_fitsio_src_and_bindgen(&self) {
        self.print_cfitsio_version_with_features(&["fitsio-src", "with-bindgen"], true);
        self.run_cargo(&[
            "nextest",
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--features",
            "fitsio-src",
            "--features",
            "bindgen",
        ]);
    }

    fn run_test_bindgen(&self) {
        self.print_cfitsio_version_with_features(&["with-bindgen"], false);
        self.run_cargo(&[
            "nextest",
            "run",
            "--manifest-path",
            "fitsio/Cargo.toml",
            "--features",
            "bindgen",
            "--no-default-features",
        ]);
    }

    fn run_test_all(&self, extra_clippy_flags: &str) {
        let tests = [
            TestType::Workspace,
            TestType::Clippy,
            TestType::FullExample,
            TestType::Array,
            TestType::FitsioSrc,
            TestType::FitsioSrcAndCmake,
            TestType::FitsioSrcAndCmakeAndBindgen,
            TestType::FitsioSrcAndBindgen,
            TestType::Bindgen,
        ];

        for test in tests {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.run_test(test, extra_clippy_flags);
            })) {
                if !self.no_fail_fast {
                    eprintln!("Test failed: {:?}", e);
                    std::panic::resume_unwind(e);
                } else {
                    eprintln!("Test failed but continuing due to --no-fail-fast");
                }
            }
        }
    }

    fn run_test(&self, test: TestType, extra_clippy_flags: &str) {
        match test {
            TestType::Workspace => self.run_test_workspace(),
            TestType::Clippy => self.run_test_clippy(extra_clippy_flags),
            TestType::FullExample => self.run_test_full_example(),
            TestType::Array => self.run_test_array(),
            TestType::FitsioSrc => self.run_test_fitsio_src(),
            TestType::FitsioSrcAndCmake => self.run_test_fitsio_src_and_cmake(),
            TestType::FitsioSrcAndCmakeAndBindgen => {
                self.run_test_fitsio_src_and_cmake_and_bindgen()
            }
            TestType::FitsioSrcAndBindgen => self.run_test_fitsio_src_and_bindgen(),
            TestType::Bindgen => self.run_test_bindgen(),
            TestType::All => self.run_test_all(extra_clippy_flags),
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    match args {
        Args::Test {
            rust_version,
            test,
            extra_clippy_flags,
            no_fail_fast,
        } => {
            let runner = TestRunner::new(rust_version, no_fail_fast);
            runner.print_preamble();
            runner.run_test(test, &extra_clippy_flags);
            ExitCode::SUCCESS
        }
    }
}
