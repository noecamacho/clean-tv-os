//! The interactive window: the launcher on a regular computer, with the keyboard as the remote.
//!
//! Arrow keys move, Enter or Space is OK, Escape or Backspace is Back, H or Home is Home, and M or the context menu key is Menu.
//! Rendering uses the same OpenGL ES 2.0 renderer as the TV (off-screen), copied to the window.

use crate::screenshot::Session;
use lume_core::app::{Effect, Key, LumeApp, Painter};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

/// Maps a keyboard key to a remote control key.
pub fn map_key(code: KeyCode) -> Option<Key> {
    Some(match code {
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => Key::Ok,
        KeyCode::Escape | KeyCode::Backspace => Key::Back,
        KeyCode::Home | KeyCode::KeyH => Key::Home,
        KeyCode::ContextMenu | KeyCode::KeyM => Key::Menu,
        _ => return None,
    })
}

struct Viewer {
    width: u32,
    height: u32,
    lume: LumeApp,
    session: Session,
    started: Instant,
    exit_after: Option<Duration>,
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    needs_frame: bool,
}

impl Viewer {
    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    fn draw(&mut self) {
        let now = self.now_ms();
        let lume = &mut self.lume;
        self.session.build(|scene| {
            let mut painter = Painter {
                list: scene.list,
                text: scene.text,
                display: *scene.display,
            };
            crate::demo::feed_stats(lume, now);
            lume.draw(&mut painter, now);
        });
        let pixels = self.session.render();
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };
        let (rgba_pixels, _) = pixels.as_chunks::<4>();
        for (target, rgba) in buffer.iter_mut().zip(rgba_pixels) {
            *target = u32::from(rgba[0]) << 16 | u32::from(rgba[1]) << 8 | u32::from(rgba[2]);
        }
        if let Err(e) = buffer.present() {
            eprintln!("cannot show the frame: {e}");
        }
        self.needs_frame = false;
    }

    fn handle(&mut self, key: Key) {
        let now = self.now_ms();
        for effect in self.lume.handle_key(key, now) {
            match effect {
                Effect::OpenApp { package, activity } => {
                    println!("open app {package} ({activity})")
                }
                Effect::OpenAndroidSettings => println!("open the Android settings"),
                Effect::OpenInputs => println!("open the TV inputs"),
                Effect::OpenAppInfo { package } => println!("open the app info of {package}"),
                Effect::Uninstall { package } => println!("uninstall {package}"),
                Effect::SaveConfig => println!("save the configuration"),
                Effect::SaveUsage => println!("save the usage history"),
                Effect::SaveKeys => println!("save the key mapping"),
                Effect::OpenTvSetting { setting } => {
                    println!("open TV setting {setting:?}")
                }
            }
        }
        self.needs_frame = true;
    }
}

impl ApplicationHandler for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("lume")
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_resizable(false);
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Rc::new(window),
            Err(e) => {
                eprintln!("cannot open a window: {e}");
                event_loop.exit();
                return;
            }
        };
        let context = softbuffer::Context::new(window.clone()).expect("a display context");
        let mut surface =
            softbuffer::Surface::new(&context, window.clone()).expect("a window surface");
        surface
            .resize(
                NonZeroU32::new(self.width).unwrap(),
                NonZeroU32::new(self.height).unwrap(),
            )
            .expect("resizing the surface");
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.needs_frame = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(key) = map_key(code) {
                        self.handle(key);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self
            .exit_after
            .is_some_and(|limit| self.started.elapsed() >= limit)
        {
            event_loop.exit();
            return;
        }
        let now = self.now_ms();
        let due = self.needs_frame || self.lume.next_deadline(now).is_some_and(|at| at <= now);
        if due {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::Poll);
            return;
        }
        // Sleep until the next change; the loop does nothing at all while idle.
        let wake = self
            .lume
            .next_deadline(now)
            .map(|at| self.started + Duration::from_millis(at));
        let limit = self.exit_after.map(|l| self.started + l);
        event_loop.set_control_flow(match (wake, limit) {
            (Some(a), Some(b)) => ControlFlow::WaitUntil(a.min(b)),
            (Some(a), None) | (None, Some(a)) => ControlFlow::WaitUntil(a),
            (None, None) => ControlFlow::Wait,
        });
    }
}

/// Opens the window and runs until it is closed. `exit_after` closes it by itself (for smoke tests).
pub fn run(
    width: u32,
    height: u32,
    lume: LumeApp,
    exit_after: Option<Duration>,
) -> Result<(), String> {
    let session = Session::new(width, height)?;
    let event_loop = EventLoop::new().map_err(|e| format!("cannot start the event loop: {e}"))?;
    let mut viewer = Viewer {
        width,
        height,
        lume,
        session,
        started: Instant::now(),
        exit_after,
        window: None,
        surface: None,
        context: None,
        needs_frame: true,
    };
    event_loop
        .run_app(&mut viewer)
        .map_err(|e| format!("the event loop stopped: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keyboard_works_as_a_remote() {
        assert_eq!(map_key(KeyCode::ArrowUp), Some(Key::Up));
        assert_eq!(map_key(KeyCode::ArrowDown), Some(Key::Down));
        assert_eq!(map_key(KeyCode::ArrowLeft), Some(Key::Left));
        assert_eq!(map_key(KeyCode::ArrowRight), Some(Key::Right));
        assert_eq!(map_key(KeyCode::Enter), Some(Key::Ok));
        assert_eq!(map_key(KeyCode::Space), Some(Key::Ok));
        assert_eq!(map_key(KeyCode::Escape), Some(Key::Back));
        assert_eq!(map_key(KeyCode::Backspace), Some(Key::Back));
        assert_eq!(map_key(KeyCode::KeyH), Some(Key::Home));
        assert_eq!(map_key(KeyCode::KeyM), Some(Key::Menu));
        assert_eq!(map_key(KeyCode::ContextMenu), Some(Key::Menu));
        assert_eq!(map_key(KeyCode::KeyA), None);
    }
}
