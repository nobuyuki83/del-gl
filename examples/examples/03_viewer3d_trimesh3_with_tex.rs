use glutin::display::GlDisplay;
use image::EncodableLayout;
use num_traits::cast::AsPrimitive;

//
use del_gl_core::gl;

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 330
precision mediump float;

layout(location=0) in vec3 position;
layout(location=1) in vec2 texIn;

uniform mat4 matMVP;

out vec2 texPrj;

void main() {
    gl_Position = matMVP * vec4(position, 1.0);
    texPrj = texIn;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 330
precision mediump float;

in vec2 texPrj;
out vec4 FragColor;
uniform sampler2D myTextureSampler;

void main() {
    // gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    FragColor = texture(myTextureSampler,texPrj);
}
\0";

pub struct MyRenderer {
    gl: gl::Gl,
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo_uv: gl::types::GLuint,
    vbo_xyz: gl::types::GLuint,
    loc_xyz: gl::types::GLint,
    loc_tex: gl::types::GLint,
    loc_mat_mvp: gl::types::GLint,
    loc_uv: gl::types::GLint,
    id_tex: gl::types::GLint,
    ebos: Vec<ElementBufferObject>,
}

struct ElementBufferObject {
    mode: gl::types::GLenum,
    elem_size: usize,
    ebo: gl::types::GLuint,
}

impl MyRenderer {
    fn new(gl: gl::Gl) -> Self {
        Self {
            gl,
            program: 0,
            vao: 0,
            vbo_xyz: 0,
            vbo_uv: 0,
            loc_tex: 0,
            loc_xyz: 0,
            loc_uv: 0,
            loc_mat_mvp: 0,
            id_tex: -1,
            ebos: vec![],
        }
    }

    fn draw(&self, transform_world2ndc: &[f32; 16]) {
        let mp0 = transform_world2ndc;
        let gl = &self.gl;
        unsafe {
            gl.UseProgram(self.program);
            gl.Uniform1i(self.loc_tex, 0);
            gl.UniformMatrix4fv(self.loc_mat_mvp, 1, gl::FALSE, mp0.as_ptr());
            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo_uv);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo_xyz);
            gl.ClearColor(0.3, 0.3, 0.3, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.Clear(gl::DEPTH_BUFFER_BIT);
            if self.id_tex >= 0 {
                gl.BindTexture(gl::TEXTURE_2D, self.id_tex.try_into().unwrap());
            }
            for ebo in &self.ebos {
                gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo.ebo);
                gl.DrawElements(
                    ebo.mode,
                    ebo.elem_size as i32,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
        }
    }

    fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    fn init_gl(&mut self) {
        unsafe {
            let gl = &self.gl;
            del_gl_core::print_info(gl);
            gl.Enable(gl::DEPTH_TEST);
            self.program =
                del_gl_core::set_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
            self.loc_xyz = del_gl_core::utility::get_attrib_location(gl, "position", self.program);
            self.loc_uv = del_gl_core::utility::get_attrib_location(gl, "texIn", self.program);
            self.loc_tex =
                del_gl_core::utility::get_uniform_location(gl, "myTextureSampler", self.program);
            self.loc_mat_mvp =
                gl.GetUniformLocation(self.program, b"matMVP\0".as_ptr() as *const _);
            assert_ne!(self.loc_xyz, -1);
            assert_ne!(self.loc_uv, -1);
            assert_ne!(self.loc_tex, -1);
            //
            self.vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut self.vao);
            gl.BindVertexArray(self.vao);
            //
            self.vbo_xyz = std::mem::zeroed();
            gl.GenBuffers(1, &mut self.vbo_xyz);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo_xyz);
            gl.BufferData(gl::ARRAY_BUFFER, 0, std::ptr::null(), gl::STATIC_DRAW);
            //
            self.vbo_uv = std::mem::zeroed();
            gl.GenBuffers(1, &mut self.vbo_uv);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo_uv);
            gl.BufferData(gl::ARRAY_BUFFER, 0, std::ptr::null(), gl::STATIC_DRAW);
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
            self.gl.DeleteBuffers(1, &self.vbo_xyz);
            self.gl.DeleteBuffers(1, &self.vbo_uv);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

pub struct MyApp {
    pub appi: del_gl_winit_glutin::app_internal::AppInternal,
    pub renderer: Option<MyRenderer>,
    pub view_rot: del_geo_core::view_rotation::Trackball<f32>,
    pub view_prj: del_geo_core::view_projection::Perspective<f32>,
    pub ui_state: del_gl_core::view_ui_state::UiState,
    pub is_left_btn_down_not_for_view_ctrl: bool,
    pub is_view_changed: bool,
}

impl MyApp {
    pub fn new(
        template: glutin::config::ConfigTemplateBuilder,
        display_builder: glutin_winit::DisplayBuilder,
    ) -> Self {
        Self {
            appi: del_gl_winit_glutin::app_internal::AppInternal::new(template, display_builder),
            renderer: None,
            ui_state: del_gl_core::view_ui_state::UiState::new(),
            view_rot: del_geo_core::view_rotation::Trackball::new(),
            view_prj: del_geo_core::view_projection::Perspective {
                lens: 24.,
                near: 0.5,
                far: 3.0,
                cam_pos: [0., 0., 2.],
                proj_direction: false,
                scale: 1.,
            },
            is_left_btn_down_not_for_view_ctrl: false,
            is_view_changed: false,
        }
    }
}

impl winit::application::ApplicationHandler for MyApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(app_state) = self.appi.resumed(event_loop) else {
            return;
        };
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        use glutin::display::GetGlDisplay;
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
        let (tri2vtx, vtx2xyz, vtx2uv) = {
            let mut obj = del_msh_core::io_obj::WavefrontObj::<usize, f32>::new();
            obj.load("asset/spot_triangulated.obj").unwrap();
            obj.unified_xyz_uv_as_trimesh()
        };
        //println!("{:?}",img.color());
        let img_tex = image::ImageReader::open("asset/spot_texture.png").unwrap();
        println!("{:?}", img_tex.format());
        let img_tex = img_tex.decode().unwrap().to_rgb8();
        let img_tex = image::imageops::flip_vertical(&img_tex);
        println!("{:?}", img_tex.dimensions());
        //
        unsafe {
            if let Some(rndr) = &mut self.renderer {
                let gl = &rndr.gl;
                gl.BindVertexArray(rndr.vao);
                gl.BindBuffer(gl::ARRAY_BUFFER, rndr.vbo_xyz);
                gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (vtx2xyz.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    vtx2xyz.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
                gl.EnableVertexAttribArray(rndr.loc_xyz as gl::types::GLuint);
                gl.VertexAttribPointer(
                    rndr.loc_xyz as gl::types::GLuint,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    3 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    std::ptr::null(),
                );
                //
                gl.BindBuffer(gl::ARRAY_BUFFER, rndr.vbo_uv);
                gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (vtx2uv.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    vtx2uv.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
                gl.EnableVertexAttribArray(rndr.loc_uv as gl::types::GLuint);
                gl.VertexAttribPointer(
                    rndr.loc_uv as gl::types::GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    2 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    std::ptr::null(),
                );
                //
                let mut ebo0 = std::mem::zeroed();
                gl.GenBuffers(1, &mut ebo0);
                gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo0);
                let tri2vtx: Vec<gl::types::GLuint> = tri2vtx.iter().map(|i| (*i).as_()).collect();
                gl.BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (tri2vtx.len() * std::mem::size_of::<gl::types::GLuint>())
                        as gl::types::GLsizeiptr,
                    tri2vtx.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
                rndr.ebos.push(ElementBufferObject {
                    mode: gl::TRIANGLES,
                    elem_size: tri2vtx.len(),
                    ebo: ebo0,
                });
                //
                gl.Enable(gl::TEXTURE_2D);
                gl.ActiveTexture(gl::TEXTURE0);
                let mut id_tex: gl::types::GLuint = 0;
                gl.GenTextures(1, &mut id_tex);
                rndr.id_tex = id_tex as i32;
                gl.BindTexture(gl::TEXTURE_2D, rndr.id_tex.try_into().unwrap());
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB.try_into().unwrap(),
                    img_tex.width().try_into().unwrap(),
                    img_tex.height().try_into().unwrap(),
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img_tex.as_bytes().as_ptr() as *const _,
                );
                gl.GenerateMipmap(gl::TEXTURE_2D);
            }
        }
        assert!(self.appi.state.replace(app_state).is_none());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use glutin::prelude::GlSurface;
        self.is_left_btn_down_not_for_view_ctrl = false;
        match event {
            winit::event::WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(del_gl_winit_glutin::app_internal::AppState {
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
            winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            _ => (),
        }
        let redraw = del_gl_winit_glutin::view_navigation(
            event,
            &mut self.ui_state,
            &mut self.view_prj,
            &mut self.view_rot,
        );
        if redraw {
            if let Some(state) = &self.appi.state {
                state.window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        use glutin::prelude::GlSurface;
        if let Some(del_gl_winit_glutin::app_internal::AppState {
            gl_context,
            gl_surface,
            window,
        }) = self.appi.state.as_ref()
        {
            let img_shape = { (window.inner_size().width, window.inner_size().height) };
            let cam_model = self.view_rot.mat4_col_major();
            let cam_projection = self
                .view_prj
                .mat4_col_major(img_shape.0 as f32 / img_shape.1 as f32);
            let transform_world2ndc =
                del_geo_core::mat4_col_major::mult_mat_col_major(&cam_projection, &cam_model);
            //
            let renderer = self.renderer.as_ref().unwrap();
            renderer.draw(&transform_world2ndc);
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");
        self.appi.suspended();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window_attributes = winit::window::Window::default_attributes()
        .with_transparent(false)
        .with_title("02_meshtex")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: 600,
            height: 600,
        });
    let template = glutin::config::ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));
    let display_builder =
        glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes));
    let mut app = MyApp::new(template, display_builder);
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    app.appi.exit_state
}
