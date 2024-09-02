use glutin::config::ConfigTemplateBuilder;
use glutin::display::GetGlDisplay;
use glutin::display::GlDisplay;
use glutin::prelude::GlSurface;
use glutin_winit::DisplayBuilder;
use image::EncodableLayout;
use std::error::Error;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::ops::Deref;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
//
use del_gl::gl::types::GLfloat;
use del_gl::{app_internal, gl};

pub unsafe fn gen_texture(
    gl: &gl::Gl,
    width: gl::types::GLsizei,
    height: gl::types::GLsizei,
    data: &[u8],
    format: gl::types::GLenum,
) -> gl::types::GLuint {
    gl.Enable(gl::TEXTURE_2D);
    gl.ActiveTexture(gl::TEXTURE0);
    let mut id_tex: gl::types::GLuint = 0;
    gl.GenTextures(1, &mut id_tex);
    gl.BindTexture(gl::TEXTURE_2D, id_tex);
    gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl.TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGB.try_into().unwrap(),
        width,
        height,
        0,
        format,
        gl::UNSIGNED_BYTE,
        data.as_ptr() as *const _,
    );
    gl.GenerateMipmap(gl::TEXTURE_2D);
    id_tex
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 330
precision mediump float;

layout(location=0) in vec2 position;
layout(location=1) in vec2 texIn;
out vec2 texPrj;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texPrj = texIn;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 330
precision mediump float;

in vec2 texPrj;
uniform sampler2D myTextureSampler;

void main() {
    // gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    gl_FragColor = texture(myTextureSampler,texPrj);
}
\0";

pub struct MyRenderer {
    gl: gl::Gl,
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    loc_tex: gl::types::GLint,
    id_tex: gl::types::GLuint,
}

impl MyRenderer {
    fn new<D: GlDisplay>(gl_display: &D) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            Self {
                gl,
                program: 0,
                vao: 0,
                vbo: 0,
                loc_tex: 0,
                id_tex: 0,
            }
        }
    }

    fn draw(&self) {
        self.draw_with_clear_color(0.1, 0.1, 0.1, 0.3)
    }

    fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    fn init_gl(&mut self) {
        unsafe {
            let gl = &self.gl;
            if let Some(renderer) = del_gl::get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = del_gl::get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }
            if let Some(shaders_version) = del_gl::get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION)
            {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }
            self.program =
                del_gl::set_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
            self.vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut self.vao);
            gl.BindVertexArray(self.vao);
            self.vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut self.vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl.BufferData(gl::ARRAY_BUFFER, 0, 0 as *const _, gl::STATIC_DRAW);
            let loc_pos = gl.GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
            let loc_uv = gl.GetAttribLocation(self.program, b"texIn\0".as_ptr() as *const _);
            let loc_tex =
                gl.GetUniformLocation(self.program, b"myTextureSampler".as_ptr() as *const _);
            gl.VertexAttribPointer(
                loc_pos as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                4 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                loc_uv as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                4 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(loc_pos as gl::types::GLuint);
            gl.EnableVertexAttribArray(loc_uv as gl::types::GLuint);
            //
            gl.Enable(gl::TEXTURE_2D);
            gl.ActiveTexture(gl::TEXTURE0);
            let mut id_tex: gl::types::GLuint = 0;
            gl.GenTextures(1, &mut id_tex);
            gl.BindTexture(gl::TEXTURE_2D, id_tex);
            self.id_tex = id_tex;
        }
    }
}

impl MyRenderer {
    fn draw_with_clear_color(&self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        unsafe {
            self.gl.UseProgram(self.program);
            self.gl.Uniform1i(self.loc_tex, 0);
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.ClearColor(red, green, blue, alpha);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

impl Deref for MyRenderer {
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
            let mut render: MyRenderer = MyRenderer::new(&app_state.gl_context.display());
            render.init_gl();
            render
        });
        unsafe {
            #[rustfmt::skip]
            static VERTEX_DATA: [f32; 24] = [
                -1.0, -1.0, 0., 0.,
                1.0, -1.0, 1., 0.,
                1.0, 1.0, 1., 1.,
                //
                -1.0, -1.0, 0., 0.,
                1.0, 1.0, 1., 1.,
                -1.0, 1.0, 0., 1.
            ];
            //println!("{:?}",img.color());
            let img = image::ImageReader::open("asset/spot_texture.png").unwrap();
            println!("{:?}", img.format());
            let img = img.decode().unwrap().to_rgb8();
            let img = image::imageops::flip_vertical(&img);
            println!("{:?}", img.dimensions());
            //
            if let Some(rndr) = &self.renderer {
                let gl = &rndr.gl;
                gl.BindBuffer(gl::ARRAY_BUFFER, rndr.vbo);
                gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    VERTEX_DATA.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
                gl.BindTexture(gl::TEXTURE_2D, rndr.id_tex);
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB.try_into().unwrap(),
                    img.width().try_into().unwrap(),
                    img.height().try_into().unwrap(),
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img.as_bytes().as_ptr() as *const _,
                );
                gl.GenerateMipmap(gl::TEXTURE_2D);
            }
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
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
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

fn main() -> Result<(), Box<dyn Error>> {
    let window_attributes = Window::default_attributes()
        .with_transparent(false)
        .with_title("Glutin triangle gradient example (press Escape to exit)");
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));
    let mut app = MyApp::new(template, display_builder);
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    app.appi.exit_state
}
