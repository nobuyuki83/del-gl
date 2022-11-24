use glutin::event_loop::EventLoop;
use glutin::ContextBuilder;

use crate::gl;

pub struct OffScreenRender {
    pub gl: gl::Gles2,
    pub width: u32,
    pub height: u32
}

impl OffScreenRender {
    pub fn new(width: u32, height: u32) -> Self {
        let el = EventLoop::new();
        let win_size = glutin::dpi::PhysicalSize::new(width, height);
        let headless_context = ContextBuilder::new().build_headless(&el, win_size).unwrap();
        let headless_context = unsafe { headless_context.make_current().unwrap() };
        let gl = gl::Gl::load_with(|ptr| headless_context.get_proc_address(ptr) as *const _);
        OffScreenRender {
            width: width,
            height: height,
            gl: gl
        }
    }

    pub unsafe fn start(&self) {
        let mut fbo: gl::types::GLuint = 0;// fbo, render_buf;
        let mut render_buf: gl::types::GLuint = 0;
        let gl = &self.gl;
        gl.Viewport(0, 0, self.width as _, self.height as _);
        gl.GenFramebuffers(1, &mut fbo);
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);
        //
        gl.GenRenderbuffers(1, &mut render_buf);
        gl.BindRenderbuffer(gl::RENDERBUFFER, render_buf);
        gl.RenderbufferStorage(gl::RENDERBUFFER, gl::RGB, self.width as _, self.height as _);
        gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
        //
        gl.FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, render_buf);
        gl.ClearColor(0.8, 0.8, 1.0, 1.0);
        gl.Clear(gl::COLOR_BUFFER_BIT);
    }

    pub unsafe fn save(&self) -> Vec<u8> {
        let mut data = vec!(0_u8; (self.width * self.height * 3) as _);
        self.gl.ReadBuffer(gl::COLOR_ATTACHMENT0);
        self.gl.ReadPixels(
            0, 0, self.width as _, self.height as _, gl::RGB, gl::UNSIGNED_BYTE,
            data.as_mut_ptr() as *mut std::ffi::c_void);
        data
    }
}