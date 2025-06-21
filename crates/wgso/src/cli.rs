#![allow(clippy::print_stdout, clippy::use_debug)]

use crate::runner::gpu;
use crate::{Program, Runner};
use clap::Parser;
use futures::channel::oneshot::{Receiver, Sender};
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::WindowId;
// coverage: off (not easy to test)

#[cfg(target_os = "android")]
pub(crate) static ANDROID_APP: std::sync::OnceLock<android_activity::AndroidApp> =
    std::sync::OnceLock::new();

/// Arguments of `wgso` CLI.
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub enum Args {
    /// Install dependencies of a WGSO program.
    Install(InstallArgs),
    /// Run a WGSO program.
    Run(RunArgs),
    /// Display the analysis result of a parsed WGSO program.
    Analyze(AnalyzeArgs),
}

impl Args {
    /// Runs CLI depending on provided arguments.
    pub fn run(self) {
        match self {
            Self::Install(args) => args.run(),
            Self::Run(args) => args.run(),
            Self::Analyze(args) => args.run(),
        }
    }
}

#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct InstallArgs {
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
                    exit_on_error(format!(
                        "Cannot clear {} folder: {error}",
                        dep_folder_path.display()
                    ));
                }
            }
        }
        if let Err(error) = wgso_deps::retrieve_dependencies(self.path.join("wgso.yaml")) {
            exit_on_error(error);
        }
    }
}

#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct RunArgs {
    /// Path to the WGSO program directory to run.
    pub path: PathBuf,
    /// List of GPU buffers to display at each step.
    #[arg(short, long, num_args(0..), default_values_t = Vec::<String>::new())]
    pub buffer: Vec<String>,
    /// Print FPS in standard output.
    #[clap(long, short, action)]
    pub fps: bool,
}

impl RunArgs {
    const DEFAULT_SIZE: (u32, u32) = (800, 600);

    fn run(self) {
        let path = self.path.clone();
        let mut runner = WindowRunner::new(self, move |event_loop, sender| {
            let window = gpu::create_window(event_loop, Self::DEFAULT_SIZE);
            sender
                .send(Runner::new(path.as_path(), Some(window), None))
                .expect("Cannot send created runner");
        });
        EventLoop::builder()
            .build()
            .expect("event loop initialization failed")
            .run_app(&mut runner)
            .expect("event loop failed");
    }

    /// Runs a WGSO program on Web.
    #[cfg(target_arch = "wasm32")]
    pub fn run_web(self, source: impl crate::SourceFolder + Send + 'static) {
        use winit::platform::web::{EventLoopExtWebSys, WindowExtWebSys};
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let _ = console_log::init_with_level(log::Level::Info);
        let runner = WindowRunner::new(self, move |event_loop, sender| {
            let window = gpu::create_window(event_loop, Self::DEFAULT_SIZE);
            if let Some(canvas) = window.canvas() {
                canvas.set_id("wgso");
                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| doc.body())
                    .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
                    .expect("cannot append canvas to document body");
            }
            let source = source.clone();
            wasm_bindgen_futures::spawn_local(async move {
                sender
                    .send(Runner::new_async(source, Some(window), None).await)
                    .expect("Cannot send created runner");
            });
        });
        EventLoop::builder()
            .build()
            .expect("event loop initialization failed")
            .spawn_app(runner);
    }

    /// Runs a WGSO program on Android.
    #[cfg(target_os = "android")]
    pub fn run_android(
        self,
        android_app: android_activity::AndroidApp,
        source: impl crate::SourceFolder + Send + 'static,
    ) {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let mut runner = WindowRunner::new(self, move |event_loop, sender| {
            let window = gpu::create_window(event_loop, Self::DEFAULT_SIZE);
            sender
                .send(Runner::new(source.clone(), Some(window), None))
                .expect("Cannot send created runner");
        });
        ANDROID_APP.get_or_init(|| android_app.clone());
        EventLoop::builder()
            .with_android_app(android_app)
            .build()
            .expect("event loop initialization failed")
            .run_app(&mut runner)
            .expect("event loop failed");
    }
}

#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct AnalyzeArgs {
    /// Path to the WGSO program directory to analyze.
    path: String,
}

impl AnalyzeArgs {
    #[allow(clippy::similar_names)]
    fn run(self) {
        match Runner::new(Path::new(&self.path), None, None) {
            Ok(runner) => println!("{runner:#?}"),
            Err(program) => exit_on_error(program.render_errors()),
        }
    }
}

struct WindowRunner {
    args: RunArgs,
    #[allow(clippy::type_complexity)]
    create_runner_fn: Box<dyn Fn(&ActiveEventLoop, Sender<Result<Runner, Program>>)>,
    runner: Option<Runner>,
    runner_receiver: Option<Receiver<Result<Runner, Program>>>,
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
        if let Some(receiver) = &mut self.runner_receiver {
            if let Ok(Some(runner)) = receiver.try_recv() {
                match runner {
                    Ok(runner) => self.runner = Some(runner),
                    Err(program) => exit_on_error(program.render_errors()),
                }
                self.runner_receiver = None;
            }
        }
        if let Some(runner) = &mut self.runner {
            match event {
                WindowEvent::RedrawRequested => self.update(),
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => self.update_window_size(size),
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        runner.std_state.keyboard.update_key(key, event.state);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    runner.std_state.mouse.update_position(position);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    runner.std_state.mouse.update_button(button, state);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    runner.std_state.mouse.update_wheel_delta(delta);
                }
                _ => (),
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(runner) = &mut self.runner {
            if let DeviceEvent::MouseMotion { delta } = event {
                runner.std_state.mouse.update_delta(delta);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.request_redraw();
        }
    }
}

impl WindowRunner {
    fn new(
        args: RunArgs,
        create_runner_fn: impl Fn(&ActiveEventLoop, Sender<Result<Runner, Program>>) + 'static,
    ) -> Self {
        Self {
            args,
            create_runner_fn: Box::new(create_runner_fn),
            runner: None,
            runner_receiver: None,
        }
    }

    fn refresh_surface(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.refresh_surface();
        } else {
            let (sender, receiver) = futures::channel::oneshot::channel();
            self.runner_receiver = Some(receiver);
            (self.create_runner_fn)(event_loop, sender);
        }
    }

    fn update(&mut self) {
        if let Some(runner) = &mut self.runner {
            if let Err(program) = runner.reload_on_change() {
                println!("{}", program.render_errors());
            }
            if let Err(program) = runner.run_step() {
                exit_on_error(program.render_errors());
            }
            if self.args.fps {
                println!("FPS: {}", (1. / runner.delta_secs()).round());
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

fn exit_on_error(error: impl Display) {
    println!("{error}");
    #[cfg(not(target_arch = "wasm32"))]
    std::process::exit(1);
    #[cfg(target_arch = "wasm32")]
    log::error!("{error}");
}
