//! Play a Fablespace project.
//!
//! `fablespace-runtime --project sample` opens the game. With no project, the
//! executable looks for `data/game.json` beside itself.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fablespace_data::{resolve, Direction};
use fablespace_runtime::{draw, gpu, self_test, Input, Screen, Session};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));
    let paths = match resolve(args.project.as_deref(), exe_dir) {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    if args.self_test {
        match self_test(paths) {
            Ok(()) => {
                println!("self-test ok");
                return;
            }
            Err(err) => {
                eprintln!("self-test failed: {err}");
                std::process::exit(1);
            }
        }
    }
    let session = match Session::boot(paths) {
        Ok(session) => session,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    let event_loop = EventLoop::new().unwrap_or_else(|err| {
        eprintln!("window: {err}");
        std::process::exit(1);
    });
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        session,
        window: None,
        gpu: None,
        keys: Keys::default(),
        edges: Input::default(),
        dir_edge: None,
        smoke: if args.gpu_smoke { Some(40) } else { None },
        failed: false,
    };
    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("window: {err}");
        std::process::exit(1);
    }
    if app.failed {
        std::process::exit(1);
    }
}

struct Args {
    project: Option<PathBuf>,
    self_test: bool,
    gpu_smoke: bool,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut project = None;
        let mut self_test = false;
        let mut gpu_smoke = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--project" => {
                    project = Some(PathBuf::from(
                        args.next().ok_or("missing path after --project")?,
                    ));
                }
                "--self-test" => self_test = true,
                "--gpu-smoke" => gpu_smoke = true,
                "--help" | "-h" => {
                    println!("fablespace-runtime [--project PATH]");
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument {other}")),
            }
        }
        Ok(Self {
            project,
            self_test,
            gpu_smoke,
        })
    }
}

struct App {
    session: Session,
    window: Option<Arc<Window>>,
    gpu: Option<gpu::Gpu>,
    keys: Keys,
    edges: Input,
    dir_edge: Option<Direction>,
    smoke: Option<u32>,
    failed: bool,
}

#[derive(Default)]
struct Keys {
    dirs: Vec<Direction>,
    dash: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let title = self.session.game.system.game_title.clone();
        let attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::PhysicalSize::new(1088u32, 832u32));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                eprintln!("create window: {err}");
                self.failed = true;
                event_loop.exit();
                return;
            }
        };
        match pollster::block_on(gpu::Gpu::new(window.clone())) {
            Ok(gpu) => self.gpu = Some(gpu),
            Err(err) => {
                eprintln!("{err}");
                self.failed = true;
                event_loop.exit();
                return;
            }
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::Focused(false) => {
                self.keys = Keys::default();
            }
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event),
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(16),
            ));
            return;
        }
        let mut input = Input {
            dir: None,
            confirm: self.edges.confirm,
            cancel: self.edges.cancel,
            menu: self.edges.menu,
            dash: self.keys.dash,
        };
        self.edges = Input::default();
        let modal = self.session.message.is_some()
            || self.session.choice.is_some()
            || self.session.battle.is_some()
            || self.session.menu.is_some()
            || self.session.save_ui.is_some()
            || self.session.screen != Screen::Map;
        if modal {
            input.dir = self.dir_edge.take();
        } else {
            self.dir_edge = None;
            input.dir = self.keys.dirs.last().copied();
        }
        self.session.update(&input);
        if self.session.quit {
            event_loop.exit();
            return;
        }
        if let Some(left) = self.smoke.as_mut() {
            *left = left.saturating_sub(1);
            if *left == 0 {
                event_loop.exit();
                return;
            }
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));
    }
}

impl App {
    fn on_key(&mut self, event: winit::event::KeyEvent) {
        if event.repeat {
            return;
        }
        let pressed = event.state == ElementState::Pressed;
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        if let Some(dir) = dir_of(code) {
            self.keys.dirs.retain(|held| *held != dir);
            if pressed {
                self.keys.dirs.push(dir);
                self.dir_edge = Some(dir);
            }
            return;
        }
        match code {
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.keys.dash = pressed,
            KeyCode::KeyZ | KeyCode::Enter | KeyCode::Space => {
                if pressed {
                    self.edges.confirm = true;
                }
            }
            KeyCode::KeyX | KeyCode::Backspace => {
                if pressed {
                    self.edges.cancel = true;
                }
            }
            KeyCode::Escape => {
                if pressed {
                    let modal = self.session.message.is_some()
                        || self.session.choice.is_some()
                        || self.session.battle.is_some()
                        || self.session.menu.is_some()
                        || self.session.save_ui.is_some()
                        || self.session.screen != Screen::Map;
                    if modal {
                        self.edges.cancel = true;
                    } else {
                        self.edges.menu = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn redraw(&mut self) {
        let (Some(window), Some(gpu)) = (self.window.as_ref(), self.gpu.as_mut()) else {
            return;
        };
        let size = window.inner_size();
        let paths = self.session.paths.clone();
        let sprites = draw::frame(&self.session, &mut |key| gpu.size(&paths, key));
        gpu.draw(
            &sprites,
            size.width,
            size.height,
            draw::VIEW_W,
            draw::VIEW_H,
        );
    }
}

fn dir_of(code: KeyCode) -> Option<Direction> {
    match code {
        KeyCode::ArrowUp | KeyCode::KeyW => Some(Direction::Up),
        KeyCode::ArrowDown | KeyCode::KeyS => Some(Direction::Down),
        KeyCode::ArrowLeft | KeyCode::KeyA => Some(Direction::Left),
        KeyCode::ArrowRight | KeyCode::KeyD => Some(Direction::Right),
        _ => None,
    }
}
