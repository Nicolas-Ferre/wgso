//! WGSO CLI.
#![allow(clippy::print_stdout, clippy::use_debug)]

use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use std::{fs, process};
use wgso::{Program, Runner};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

// coverage: off (not easy to test)

fn main() {
    Args::parse().run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
enum Args {
    /// Install dependencies of a WGSO program.
    Install(InstallArgs),
    /// Run a WGSO program.
    Run(RunArgs),
    /// Display the analysis result of a parsed WGSO program.
    Analyze(AnalyzeArgs),
}

impl Args {
    fn run(self) {
        match self {
            Self::Install(args) => args.run(),
            Self::Run(args) => args.run(),
            Self::Analyze(args) => args.run(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct InstallArgs {
    /// Path to the WGSO program directory containing a `wgso.yaml` file.
    path: PathBuf,
    /// Force retrieval of all dependencies, even if they have already been retrieved.
    #[clap(long, short, action)]
    force: bool,
}

impl InstallArgs {
    fn run(self) {
        if self.force {
            let dep_folder_path = self.path.join("_");
            if dep_folder_path.is_dir() {
                if let Err(error) = fs::remove_dir_all(&dep_folder_path) {
                    println!("Cannot clear {} folder: {error}", dep_folder_path.display());
                    process::exit(1);
                }
            }
        }
        if let Err(error) = wgso_deps::retrieve_dependencies(self.path.join("wgso.yaml")) {
            println!("{error}");
            process::exit(1);
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct RunArgs {
    /// Path to the WGSO program directory to run.
    path: PathBuf,
    /// List of GPU buffers to display at each step.
    #[arg(short, long, num_args(0..), default_values_t = Vec::<String>::new())]
    buffer: Vec<String>,
    /// Print FPS in standard output.
    #[clap(long, short, action)]
    fps: bool,
}

impl RunArgs {
    fn run(self) {
        let mut runner = WindowRunner {
            args: self,
            runner: None,
            last_instant: Instant::now(),
        };
        EventLoop::builder()
            .build()
            .expect("event loop initialization failed")
            .run_app(&mut runner)
            .expect("event loop failed");
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
        match Runner::new(&self.path, None, None) {
            Ok(runner) => println!("{runner:#?}"),
            Err(program) => exit_on_error(&program),
        }
    }
}
#[derive(Debug)]
struct WindowRunner {
    args: RunArgs,
    runner: Option<Runner>,
    last_instant: Instant,
}

impl ApplicationHandler for WindowRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.refresh_surface(event_loop);
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => self.update(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.update_window_size(size),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.request_redraw();
        }
    }
}

impl WindowRunner {
    fn refresh_surface(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.refresh_surface();
        } else {
            match Runner::new(&self.args.path, Some(event_loop), None) {
                Ok(runner) => self.runner = Some(runner),
                Err(program) => exit_on_error(&program),
            }
        }
    }

    fn update(&mut self) {
        if let Some(runner) = &mut self.runner {
            if let Err(program) = runner.reload_on_change() {
                println!("{}", program.render_errors());
            }
            if let Err(program) = runner.run_step() {
                exit_on_error(program);
            }
            let delta = self.last_instant.elapsed();
            self.last_instant = Instant::now();
            if self.args.fps {
                println!("FPS: {}", (1. / delta.as_secs_f32()).round());
            }
            for buffer in &self.args.buffer {
                println!("Buffer `{buffer}`: {:?}", runner.read(buffer));
            }
        }
    }

    fn update_window_size(&mut self, size: PhysicalSize<u32>) {
        if let Some(runner) = &mut self.runner {
            runner.update_surface_size(size);
        }
    }
}

fn exit_on_error(program: &Program) {
    println!("{}", program.render_errors());
    process::exit(1);
}
