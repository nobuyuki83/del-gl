use crate::gl;

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 330

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

in vec2 texPrj;
out vec4 FragColor;
uniform sampler2D myTextureSampler;

void main() {
    // FragColor = vec4(1.0, 0.0, 1.0, 1.0);
    FragColor = texture(myTextureSampler,texPrj);
}
\0";

pub struct Drawer {
    pub gl: gl::Gl,
    pub program: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub vbo: gl::types::GLuint,
    pub loc_tex: gl::types::GLint,
    pub id_tex: gl::types::GLuint,
}

impl Drawer {
    pub fn new(gl0: gl::Gl) -> Self {
        Self {
            gl: gl0,
            program: 0,
            vao: 0,
            vbo: 0,
            loc_tex: 0,
            id_tex: 0,
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.UseProgram(self.program);
            self.gl.Uniform1i(self.loc_tex, 0);
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.ClearColor(0.8, 0.8, 0.8, 0.3);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    pub fn init_gl(&mut self) {
        unsafe {
            let gl = &self.gl;
            crate::print_info(gl);
            self.program =
                crate::set_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
            let loc_xyz = crate::utility::get_attrib_location(gl, "position", self.program);
            let loc_uv = crate::utility::get_attrib_location(gl, "texIn", self.program);
            let loc_tex =
                crate::utility::get_uniform_location(gl, "myTextureSampler", self.program);
            assert_ne!(loc_xyz, -1);
            assert_ne!(loc_uv, -1);
            assert_ne!(loc_tex, -1);
            //
            self.vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut self.vao);
            gl.BindVertexArray(self.vao);
            self.vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut self.vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            // gl.BufferData(gl::ARRAY_BUFFER, 0, 0 as *const _, gl::STATIC_DRAW);
            dbg!(self.program, loc_xyz, loc_uv, loc_tex);
            gl.VertexAttribPointer(
                loc_xyz as gl::types::GLuint,
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
            gl.EnableVertexAttribArray(loc_xyz as gl::types::GLuint);
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

impl std::ops::Deref for Drawer {
    type Target = gl::Gl;
    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for Drawer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
