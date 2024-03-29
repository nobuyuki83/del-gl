//! class for handling glutin window with 3D navigation

use crate::gl;

use glutin::event::{ElementState, Event, MouseButton, WindowEvent};
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

use crate::nalgebra::view_navigation3::Navigation3;
use crate::view_ui_state::UiState;

pub struct Viewer3 {
    pub windowed_context: ContextWrapper<PossiblyCurrent,glutin::window::Window>,
    pub gl: gl::Gles2,
    pub nav: Navigation3,
    pub ui_state: UiState,
    pub should_draw: bool,
    pub should_close: bool,
    pub is_left_btn_down_not_for_view_ctrl: bool,
    pub is_view_changed: bool
}

impl Viewer3 {
    pub fn open() -> (Self, EventLoop<()>) {
        let el = EventLoop::new();
        let wb = WindowBuilder::new().with_title("A fantastic window!");
        let wc0 = ContextBuilder::new().build_windowed(wb, &el).unwrap();
        let windowed_context = unsafe { wc0.make_current().unwrap() };

        let gl = gl::Gl::load_with(|ptr| windowed_context.context().get_proc_address(ptr) as *const _);

        (Viewer3 {
            windowed_context: windowed_context,
            gl: gl,
            ui_state: UiState::new(),
            nav: Navigation3::new(1.) ,
            should_close: false,
            should_draw: false,
            is_view_changed: false,
            is_left_btn_down_not_for_view_ctrl: false
        },
         el)
    }

    pub fn get_opengl_version(&self) -> String {
        let version = unsafe {
            let data = std::ffi::CStr::from_ptr(self.gl.GetString(gl::VERSION) as *const _)
                .to_bytes().to_vec();
            String::from_utf8(data).unwrap()
        };
        version
    }

    pub fn event_handle(&mut self, event:&Event<()>) {
        self.should_draw = false;
        self.should_draw = false;
        self.is_left_btn_down_not_for_view_ctrl = false;
        self.is_view_changed = false;
        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event: win_event, .. } => match win_event {
                WindowEvent::CloseRequested => { self.should_close = true; },
                WindowEvent::Resized(physical_size) => {
                    self.windowed_context.resize(*physical_size);
                    self.ui_state.win_height = physical_size.height;
                    self.ui_state.win_width = physical_size.width;
                },
                WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                    if *button == MouseButton::Left && *state == ElementState::Pressed {
                        self.ui_state.is_left_btn = true;
                        if (!self.ui_state.is_mod_shift) && (!self.ui_state.is_mod_alt) {
                            self.is_left_btn_down_not_for_view_ctrl = true;
                        }
                    }
                    if *button == MouseButton::Left && *state == ElementState::Released {
                        self.ui_state.is_left_btn = false;
                    }
                }
                WindowEvent::MouseWheel { device_id: _, delta, .. } => match delta {
                    glutin::event::MouseScrollDelta::LineDelta(_, dy) => {
                        self.nav.scale *= 1.01_f32.powf(*dy);
                        self.is_view_changed = false;
                        self.windowed_context.window().request_redraw();
                    }
                    _ => {}
                }
                WindowEvent::ModifiersChanged(state) => {
                    //println!("{} {}", nav.is_mod_alt, nav.is_mod_shift);
                    self.ui_state.is_mod_alt = state.alt();
                    self.ui_state.is_mod_shift = state.shift();
                }
                WindowEvent::CursorMoved { device_id: _, position, .. } => {
                    // println!("{:?} {:?} {:?}", device_id, position, modifiers);
                    self.ui_state.update_cursor_position(position.x, position.y);
                    if self.ui_state.is_left_btn && self.ui_state.is_mod_alt {
                        self.nav.camera_rotation(self.ui_state.cursor_dx, self.ui_state.cursor_dy);
                        self.is_view_changed = true;
                    }
                    if self.ui_state.is_left_btn && self.ui_state.is_mod_shift {
                        self.nav.camera_translation(
                            self.ui_state.win_width, self.ui_state.win_height,
                            self.ui_state.cursor_dx, self.ui_state.cursor_dy);
                        self.is_view_changed = true;
                    }
                    self.windowed_context.window().request_redraw();
                }
                _ => ()
            },
            Event::RedrawRequested(_window_id) => {
                unsafe {
                    self.gl.ClearColor(0.8, 0.8, 1.0, 1.0);
                    self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    self.gl.Enable(gl::DEPTH_TEST);
                    self.gl.DepthFunc(gl::LESS);
                    self.gl.Enable(gl::POLYGON_OFFSET_FILL);
                    self.gl.PolygonOffset(1.1, 4.0);
                }
                self.should_draw = true;
            },
            _ => ()
        }
    }
}