//! Window, keyboard, and the integer-scaled software framebuffer.
//!
//! The internal picture is 320x180. The window is an integer multiple of that,
//! nearest-neighbor, so the 16x16 sprites stay on the pixel grid.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod atlas;
mod draw;
mod font;
mod game;

use game::{Input, HEIGHT, WIDTH};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() {
    if let Err(err) = run() {
        eprintln!("field-battle: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct App {
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    surface_size: (u32, u32),
    keys: Keys,
    game: game::Game,
    fb: Vec<u32>,
    next_frame: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            context: None,
            surface: None,
            surface_size: (0, 0),
            keys: Keys::default(),
            game: game::Game::new(),
            fb: vec![0; (WIDTH * HEIGHT) as usize],
            next_frame: Instant::now(),
        }
    }

    fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(window) = self.window.clone() else {
            return Ok(());
        };
        let Some(surface) = self.surface.as_mut() else {
            return Ok(());
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        if self.surface_size != (size.width, size.height) {
            surface.resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )?;
            self.surface_size = (size.width, size.height);
        }
        draw::draw(&mut self.fb, &self.game);
        let mut buffer = surface.buffer_mut()?;
        blit_nearest(&mut buffer, size.width, size.height, &self.fb);
        buffer.present()?;
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Field Battle")
            .with_inner_size(PhysicalSize::new(WIDTH * 4, HEIGHT * 4))
            .with_min_inner_size(PhysicalSize::new(WIDTH, HEIGHT));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                eprintln!("field-battle: {err}");
                event_loop.exit();
                return;
            }
        };
        let context = match Context::new(window.clone()) {
            Ok(context) => context,
            Err(err) => {
                eprintln!("field-battle: {err}");
                event_loop.exit();
                return;
            }
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                eprintln!("field-battle: {err}");
                event_loop.exit();
                return;
            }
        };
        self.context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.next_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(false) => self.keys.release(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.keys
                        .on_key(code, event.state == ElementState::Pressed, event.repeat);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.present() {
                    eprintln!("field-battle: {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.clone() else {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };
        let now = Instant::now();
        if now >= self.next_frame {
            let input = self.keys.take();
            self.game.update(&input);
            self.next_frame = now + FRAME;
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
    }
}

#[derive(Default)]
struct Keys {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    tap_x: i32,
    tap_y: i32,
    confirm: bool,
    pause: bool,
}

impl Keys {
    fn release(&mut self) {
        *self = Self::default();
    }

    fn on_key(&mut self, code: KeyCode, down: bool, repeat: bool) {
        let dir = match code {
            KeyCode::ArrowLeft | KeyCode::KeyA => Some((-1, 0)),
            KeyCode::ArrowRight | KeyCode::KeyD => Some((1, 0)),
            KeyCode::ArrowUp | KeyCode::KeyW => Some((0, -1)),
            KeyCode::ArrowDown | KeyCode::KeyS => Some((0, 1)),
            _ => None,
        };
        if let Some((tx, ty)) = dir {
            match (tx, ty) {
                (-1, _) => self.left = down,
                (1, _) => self.right = down,
                (_, -1) => self.up = down,
                _ => self.down = down,
            }
            if down && !repeat {
                if tx != 0 {
                    self.tap_x = tx;
                }
                if ty != 0 {
                    self.tap_y = ty;
                }
            }
            return;
        }
        if !down || repeat {
            return;
        }
        match code {
            KeyCode::KeyZ | KeyCode::Enter | KeyCode::NumpadEnter => self.confirm = true,
            KeyCode::Escape | KeyCode::KeyP => self.pause = true,
            _ => {}
        }
    }

    fn take(&mut self) -> Input {
        let input = Input {
            held_x: axis(self.left, self.right),
            held_y: axis(self.up, self.down),
            tap_x: self.tap_x,
            tap_y: self.tap_y,
            confirm: self.confirm,
            pause: self.pause,
        };
        self.tap_x = 0;
        self.tap_y = 0;
        self.confirm = false;
        self.pause = false;
        input
    }
}

fn axis(neg: bool, pos: bool) -> i32 {
    match (neg, pos) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

fn blit_nearest(dst: &mut [u32], dst_w: u32, dst_h: u32, src: &[u32]) {
    let scale = (dst_w / WIDTH as u32).min(dst_h / HEIGHT as u32).max(1);
    let view_w = WIDTH as u32 * scale;
    let view_h = HEIGHT as u32 * scale;
    let ox = (dst_w - view_w) / 2;
    let oy = (dst_h - view_h) / 2;
    for y in 0..dst_h {
        let row = y * dst_w;
        for x in 0..dst_w {
            let color = if x >= ox && y >= oy && x < ox + view_w && y < oy + view_h {
                let sx = (x - ox) / scale;
                let sy = (y - oy) / scale;
                src[(sy * WIDTH as u32 + sx) as usize]
            } else {
                0
            };
            dst[(row + x) as usize] = color;
        }
    }
}
