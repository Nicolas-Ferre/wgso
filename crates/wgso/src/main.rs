//! WGSO CLI.
#![allow(clippy::print_stdout, clippy::use_debug)]

use clap::Parser;
use std::process;
use wgso::{Program, Runner};

// coverage: off (not simple to test)

fn main() {
    Args::parse().run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
enum Args {
    /// Run a WGSO program.
    Run(RunArgs),
    /// Display the analysis result of a parsed WGSO program.
    Analyze(AnalyzeArgs),
}

impl Args {
    fn run(self) {
        match self {
            Self::Run(args) => args.run(),
            Self::Analyze(args) => args.run(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct RunArgs {
    /// Path to the WGSO program directory to run.
    path: String,
    /// List of GPU buffers to display at each step.
    #[arg(short, long, num_args(0..), default_values_t = Vec::<String>::new())]
    buffer: Vec<String>,
    /// Number of steps to run (0 to run indefinitely).
    #[arg(short, long, default_value_t = 0)]
    steps: u32,
}

impl RunArgs {
    fn run(self) {
        match Runner::new(&self.path) {
            Ok(runner) => {
                if self.steps == 0 {
                    loop {
                        self.run_step(&runner);
                    }
                } else {
                    for _ in 0..self.steps {
                        self.run_step(&runner);
                    }
                }
            }
            Err(program) => trigger_errors(program),
        }
    }

    fn run_step(&self, runner: &Runner) {
        for buffer in &self.buffer {
            println!("Buffer `{buffer}`: {:?}", runner.read(buffer));
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct AnalyzeArgs {
    /// Path to the WGSO program directory to analyze.
    path: String,
}

impl AnalyzeArgs {
    #[allow(clippy::similar_names)]
    fn run(self) {
        match Runner::new(&self.path) {
            Ok(runner) => println!("{runner:#?}"),
            Err(program) => trigger_errors(program),
        }
    }
}

fn trigger_errors(program: Program) {
    for error in &program.errors {
        println!("{}", error.render(&program));
    }
    process::exit(1);
}
