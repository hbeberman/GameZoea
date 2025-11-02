pub use crate::emu::cpu::*;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture, wgpu::PresentMode};
use std::{
    collections::VecDeque,
    process,
    sync::{Arc, mpsc::SyncSender},
    thread,
};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;
pub const MAX_SCALE: u32 = 5;

pub type FrameSender = SyncSender<Vec<u8>>;

#[derive(Debug)]
pub enum WindowMessage {
    Frame(Vec<u8>),
}

pub fn create_frame_channel() -> (FrameSender, std::sync::mpsc::Receiver<Vec<u8>>) {
    std::sync::mpsc::sync_channel(2)
}

pub fn run(scale: u32, frame_rx: std::sync::mpsc::Receiver<Vec<u8>>) -> Result<(), EventLoopError> {
    let mut event_loop_builder = EventLoop::<WindowMessage>::with_user_event();

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
    let proxy = event_loop.create_proxy();

    let _notifier = thread::spawn(move || {
        while let Ok(frame) = frame_rx.recv() {
            if proxy.send_event(WindowMessage::Frame(frame)).is_err() {
                break;
            }
        }
    });

    let mut app = WindowApp::new(scale);
    event_loop.run_app(&mut app)
}

struct WindowApp {
    scale: u32,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    frame_queue: VecDeque<Vec<u8>>,
}

impl WindowApp {
    fn new(scale: u32) -> Self {
        let safe_scale = scale.clamp(1, MAX_SCALE);

        debug_assert_eq!(
            safe_scale, scale,
            "Scale {scale} is outside the supported range 1..={MAX_SCALE}"
        );

        Self {
            scale: safe_scale,
            window: None,
            pixels: None,
            frame_queue: VecDeque::new(),
        }
    }

    fn scaled_dimensions(&self) -> (u32, u32) {
        (SCREEN_WIDTH * self.scale, SCREEN_HEIGHT * self.scale)
    }
}

impl ApplicationHandler<WindowMessage> for WindowApp {
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

        let pixels = match PixelsBuilder::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)
            .present_mode(PresentMode::Fifo)
            .build()
        {
            Ok(pixels) => pixels,
            Err(err) => {
                eprintln!("failed to create pixel surface: {err}");
                event_loop.exit();
                return;
            }
        };

        window.request_redraw();

        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window_arc = match self.window.as_ref() {
            Some(window) if window.id() == window_id => Arc::clone(window),
            _ => return,
        };

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
                let window = &window_arc;
                if size.width == 0 || size.height == 0 {
                    return;
                }

                let (desired_width, desired_height) = self.scaled_dimensions();
                if size.width != desired_width || size.height != desired_height {
                    let _ =
                        window.request_inner_size(PhysicalSize::new(desired_width, desired_height));
                    return;
                }

                if let Some(pixels) = self.pixels.as_mut()
                    && let Err(err) = pixels.resize_surface(size.width, size.height)
                {
                    eprintln!("failed to resize surface: {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(pixels) = self.pixels.as_mut() else {
                    return;
                };

                let Some(frame) = self.frame_queue.pop_front() else {
                    return;
                };

                if pixels.frame().len() != frame.len() {
                    eprintln!(
                        "frame size mismatch: window={} incoming={}",
                        pixels.frame().len(),
                        frame.len()
                    );
                    return;
                }

                pixels.frame_mut().copy_from_slice(&frame);

                if let Err(err) = pixels.render() {
                    eprintln!("failed to render frame: {err}");
                }

                if let Some(window) = self.window.as_ref()
                    && !self.frame_queue.is_empty()
                {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: WindowMessage) {
        match event {
            WindowMessage::Frame(frame) => {
                self.frame_queue.push_back(frame);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }

        if !self.frame_queue.is_empty()
            && let Some(window) = self.window.as_ref()
        {
            window.request_redraw();
        }

        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
