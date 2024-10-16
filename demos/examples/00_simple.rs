use glutin::config::ConfigTemplateBuilder;
use glutin::display::GetGlDisplay;
use glutin::display::GlDisplay;
use glutin::prelude::GlSurface;
use glutin_winit::DisplayBuilder;
use winit::application::ApplicationHandler;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

//
use del_gl_core::gl;
use del_gl_winit_glutin::app_internal;

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 330

in vec2 xyzIn;
in vec3 rgbIn;
out vec3 v_color;

void main() {
    gl_Position = vec4(xyzIn, 0.0, 1.0);
    v_color = rgbIn;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 330

in vec3 v_color;
out vec4 FragColor;

void main() {
    FragColor = vec4(v_color, 1.0);
}
\0";

pub struct MyRenderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    gl: gl::Gl,
}

impl MyRenderer {
    fn new(gl: gl::Gl) -> Self {
        Self {
            program: 0,
            vao: 0,
            vbo: 0,
            gl,
        }
    }

    fn init_gl(&mut self) {
        let gl = &self.gl;
        unsafe {
            self.program =
                del_gl_core::set_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
            self.vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut self.vao);
            gl.BindVertexArray(self.vao);
            //
            self.vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut self.vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            //
            let loc_xyz = gl.GetAttribLocation(self.program, b"xyzIn\0".as_ptr() as *const _);
            let loc_rgb = gl.GetAttribLocation(self.program, b"rgbIn\0".as_ptr() as *const _);
            assert_ne!(loc_xyz, -1);
            assert_ne!(loc_rgb, -1);
            gl.VertexAttribPointer(
                loc_xyz as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                loc_rgb as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(loc_xyz as gl::types::GLuint);
            gl.EnableVertexAttribArray(loc_rgb as gl::types::GLuint);
        }
    }

    fn draw(&self) {
        unsafe {
            self.gl.ClearColor(0.9, 0.9, 1.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            //
            self.gl.UseProgram(self.program);
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl std::ops::Deref for MyRenderer {
    type Target = gl::Gl;
    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for MyRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

pub struct MyApp {
    pub appi: crate::app_internal::AppInternal,
    pub renderer: Option<MyRenderer>,
}

impl MyApp {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        Self {
            appi: app_internal::AppInternal::new(template, display_builder),
            renderer: None,
        }
    }
}

impl ApplicationHandler for MyApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(app_state) = self.appi.resumed(event_loop) else {
            return;
        };
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        self.renderer.get_or_insert_with(|| {
            let gl_display = &app_state.gl_context.display();
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            let mut render: MyRenderer = MyRenderer::new(gl);
            render.init_gl();
            render
        });

        let gl = if let Some(rndr) = &self.renderer {
            &rndr.gl
        } else {
            panic!();
        };

        unsafe {
            #[rustfmt::skip]
            static VERTEX_DATA: [f32; 15] = [
                -0.5, -0.5, 1.0, 0.0, 0.0,
                0.0, 0.5, 0.0, 1.0, 0.0,
                0.5, -0.5, 0.0, 0.0, 1.0,
            ];
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
        }
        assert!(self.appi.state.replace(app_state).is_none());
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");
        self.appi.suspended();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(app_internal::AppState {
                    gl_context,
                    gl_surface,
                    window: _,
                }) = self.appi.state.as_ref()
                {
                    gl_surface.resize(
                        gl_context,
                        std::num::NonZeroU32::new(size.width).unwrap(),
                        std::num::NonZeroU32::new(size.height).unwrap(),
                    );
                    let renderer = self.renderer.as_ref().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(app_internal::AppState {
            gl_context,
            gl_surface,
            window,
        }) = self.appi.state.as_ref()
        {
            let renderer = self.renderer.as_ref().unwrap();
            renderer.draw();
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));
    let display_builder = {
        let window_attributes = Window::default_attributes()
            .with_transparent(true)
            .with_title("00_simple");
        DisplayBuilder::new().with_window_attributes(Some(window_attributes))
    };
    let mut app = MyApp::new(template, display_builder);
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    app.appi.exit_state
    // del_gl::renderer::main::<Renderer>(EventLoop::new().unwrap())
}
