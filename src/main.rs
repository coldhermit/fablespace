//! Window, keyboard, and the nearest-neighbor software framebuffer.
//!
//! The picture is 1920x1080 and the window opens at that size, so one
//! framebuffer pixel is one screen pixel. A smaller or larger window still
//! samples nearest-neighbor and stays crisp.

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
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .with_min_inner_size(PhysicalSize::new(WIDTH / 4, HEIGHT / 4));
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

/// Where the picture sits in the window: origin, then width and height.
/// Integer scale when the window can hold the framebuffer. Otherwise the
/// picture shrinks to fit and the sampler stays nearest-neighbor.
fn frame_in_window(dst_w: u32, dst_h: u32) -> (u32, u32, u32, u32) {
    let src_w = WIDTH as u32;
    let src_h = HEIGHT as u32;
    if dst_w == 0 || dst_h == 0 {
        return (0, 0, 0, 0);
    }
    if dst_w >= src_w && dst_h >= src_h {
        let scale = (dst_w / src_w).min(dst_h / src_h).max(1);
        let view_w = src_w * scale;
        let view_h = src_h * scale;
        let ox = (dst_w - view_w) / 2;
        let oy = (dst_h - view_h) / 2;
        (ox, oy, view_w, view_h)
    } else if u64::from(dst_w) * u64::from(src_h) <= u64::from(dst_h) * u64::from(src_w) {
        let view_w = dst_w;
        let view_h = (u64::from(dst_w) * u64::from(src_h) / u64::from(src_w)).max(1) as u32;
        let view_h = view_h.min(dst_h);
        let oy = (dst_h - view_h) / 2;
        (0, oy, view_w, view_h)
    } else {
        let view_h = dst_h;
        let view_w = (u64::from(dst_h) * u64::from(src_w) / u64::from(src_h)).max(1) as u32;
        let view_w = view_w.min(dst_w);
        let ox = (dst_w - view_w) / 2;
        (ox, 0, view_w, view_h)
    }
}

fn blit_nearest(dst: &mut [u32], dst_w: u32, dst_h: u32, src: &[u32]) {
    let (ox, oy, view_w, view_h) = frame_in_window(dst_w, dst_h);
    let src_w = WIDTH as u32;
    let src_h = HEIGHT as u32;
    for y in 0..dst_h {
        let row = y * dst_w;
        for x in 0..dst_w {
            let color = if view_w > 0
                && view_h > 0
                && x >= ox
                && y >= oy
                && x < ox + view_w
                && y < oy + view_h
            {
                let sx = ((u64::from(x - ox) * u64::from(src_w)) / u64::from(view_w)) as u32;
                let sy = ((u64::from(y - oy) * u64::from(src_h)) / u64::from(view_h)) as u32;
                src[(sy.min(src_h - 1) * src_w + sx.min(src_w - 1)) as usize]
            } else {
                0
            };
            dst[(row + x) as usize] = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_window_is_one_to_one() {
        assert_eq!(frame_in_window(1920, 1080), (0, 0, 1920, 1080));
        let mut src = vec![0u32; (WIDTH * HEIGHT) as usize];
        src[(100 * WIDTH + 50) as usize] = 0x00FF00;
        src[(100 * WIDTH + 51) as usize] = 0x0000FF;
        let mut dst = vec![1u32; src.len()];
        blit_nearest(&mut dst, WIDTH as u32, HEIGHT as u32, &src);
        assert_eq!(dst[(100 * WIDTH + 50) as usize], 0x00FF00);
        assert_eq!(dst[(100 * WIDTH + 51) as usize], 0x0000FF);
    }

    #[test]
    fn larger_window_scales_by_an_integer() {
        assert_eq!(frame_in_window(3850, 2170), (5, 5, 3840, 2160));
    }

    #[test]
    fn smaller_window_fits_and_stays_nearest() {
        assert_eq!(frame_in_window(960, 540), (0, 0, 960, 540));
        assert_eq!(frame_in_window(1000, 1000), (0, 219, 1000, 562));
        let src = vec![0x112233u32; (WIDTH * HEIGHT) as usize];
        let mut dst = vec![0u32; 960 * 540];
        blit_nearest(&mut dst, 960, 540, &src);
        assert!(dst.iter().all(|&p| p == 0x112233));
    }
}
