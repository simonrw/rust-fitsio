use std::error::Error;
use std::process;
use std::sync::mpsc;
use std::thread;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn compile() -> Result<()> {
    process::Command::new("cargo")
        .args(&["build", "--release"])
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Error running command: {:?}", e).into())
}

fn run_rust() -> Result<Vec<f64>> {
    let output = process::Command::new("./target/release/rustversion").output()?;
    parse_lines(std::str::from_utf8(&output.stdout)?)
}

fn run_python() -> Result<Vec<f64>> {
    let output = process::Command::new("python")
        .args(&["./python_run.py"])
        .output()?;
    parse_lines(std::str::from_utf8(&output.stdout)?)
}

fn parse_lines(text: &str) -> Result<Vec<f64>> {
    Ok(text
        .split("\n")
        .filter_map(|l| {
            if l.is_empty() {
                None
            } else {
                Some(l.parse().unwrap())
            }
        })
        .collect())
}

#[derive(Debug)]
struct Stats {
    mean: f64,
    min: f64,
}

impl Stats {
    fn summarise(&self, name: &str) {
        println!("{}", name);
        println!("{:#?}", self);
        println!("");
    }
}

fn compute_stats(times: Vec<f64>) -> Stats {
    let mean = times.iter().sum::<f64>() / (times.len() as f64);
    let min = times.iter().fold(std::f64::INFINITY, |acc, v| v.min(acc));

    Stats { mean, min }
}

#[derive(Debug)]
struct ComputesStats {
    rx: mpsc::Receiver<Stats>,
    th: thread::JoinHandle<()>,
}

impl ComputesStats {
    fn new(times: Vec<f64>) -> Self {
        let (tx, rx) = mpsc::channel();

        let th = thread::spawn(move || {
            let stats = compute_stats(times);
            tx.send(stats).unwrap();
        });

        ComputesStats { rx, th }
    }

    fn stats(self) -> Stats {
        self.rx.recv().unwrap()
    }
}

fn main() -> Result<()> {
    compile()?;
    let rust_times = run_rust()?;
    let python_times = run_python()?;

    let rust_computer = ComputesStats::new(rust_times);
    let python_computer = ComputesStats::new(python_times);

    let rust_stats = rust_computer.stats();
    let python_stats = python_computer.stats();

    rust_stats.summarise("Rust");
    python_stats.summarise("Python");

    Ok(())
}
