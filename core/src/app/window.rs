pub use crate::emu::cpu::*;
use pixels::{Pixels, SurfaceTexture};
use std::{
    process,
    sync::{Arc, Mutex},
};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;
pub const MAX_SCALE: u32 = 5;

pub type SharedPixels = Arc<Mutex<Option<Pixels<'static>>>>;

pub fn create_pixels_handle() -> SharedPixels {
    Arc::new(Mutex::new(None))
}

pub fn run(scale: u32, pixels: SharedPixels) -> Result<(), EventLoopError> {
    let mut event_loop_builder = EventLoop::builder();

    #[cfg(all(target_os = "linux", feature = "wayland"))]
    {
        use winit::platform::wayland::EventLoopBuilderExtWayland;
        event_loop_builder.with_any_thread(true);
    }

    #[cfg(all(target_os = "linux", feature = "x11"))]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        event_loop_builder.with_any_thread(true);
    }

    let event_loop = event_loop_builder.build()?;
    let mut app = WindowApp::new(scale, pixels);
    event_loop.run_app(&mut app)
}

struct WindowApp {
    scale: u32,
    window: Option<Arc<Window>>,
    pixels: SharedPixels,
}

impl WindowApp {
    fn new(scale: u32, pixels: SharedPixels) -> Self {
        let safe_scale = scale.clamp(1, MAX_SCALE);

        debug_assert_eq!(
            safe_scale, scale,
            "Scale {scale} is outside the supported range 1..={MAX_SCALE}"
        );

        Self {
            scale: safe_scale,
            window: None,
            pixels,
        }
    }

    fn scaled_dimensions(&self) -> (u32, u32) {
        (SCREEN_WIDTH * self.scale, SCREEN_HEIGHT * self.scale)
    }
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let (width, height) = self.scaled_dimensions();
        let target_size = PhysicalSize::new(width, height);

        let attributes = Window::default_attributes()
            .with_title("GameZoea")
            .with_inner_size(target_size)
            .with_resizable(false);

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                eprintln!("failed to create window: {err}");
                event_loop.exit();
                return;
            }
        };

        if window.inner_size() != target_size {
            let _ = window.request_inner_size(target_size);
        }

        let surface_texture =
            SurfaceTexture::new(target_size.width, target_size.height, window.clone());

        let pixels = match Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture) {
            Ok(pixels) => pixels,
            Err(err) => {
                eprintln!("failed to create pixel surface: {err}");
                event_loop.exit();
                return;
            }
        };

        window.request_redraw();

        let mut shared_pixels = match self.pixels.lock() {
            Ok(guard) => guard,
            Err(err) => {
                eprintln!("failed to acquire pixels handle: {err}");
                event_loop.exit();
                return;
            }
        };

        *shared_pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                process::exit(0);
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } if key_event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(KeyCode::Escape) = key_event.physical_key {
                    event_loop.exit();
                    process::exit(0);
                }
                if let PhysicalKey::Code(KeyCode::KeyP) = key_event.physical_key {
                    event_loop.exit();
                    process::exit(0);
                }
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }

                let (desired_width, desired_height) = self.scaled_dimensions();
                if size.width != desired_width || size.height != desired_height {
                    let _ =
                        window.request_inner_size(PhysicalSize::new(desired_width, desired_height));
                    return;
                }

                let mut pixels_guard = match self.pixels.lock() {
                    Ok(guard) => guard,
                    Err(err) => {
                        eprintln!("failed to acquire pixels handle: {err}");
                        event_loop.exit();
                        return;
                    }
                };

                if let Some(pixels) = pixels_guard.as_mut()
                    && let Err(err) = pixels.resize_surface(size.width, size.height)
                {
                    eprintln!("failed to resize surface: {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                // Redraw requests driven by PPU
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
