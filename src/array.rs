//! draw array of position and color

use crate::gl;

pub struct Drawer {
    pub program: gl::types::GLuint,
    pub mode: gl::types::GLenum,
    pub elem_size: usize,
    pub vao: gl::types::GLuint,
    pub loc_color: gl::types::GLint,
    pub loc_mat_modelview : gl::types::GLint,
    pub loc_mat_projection: gl::types::GLint,
    pub color: [f32;3]
}

impl Drawer {
    pub fn compile_shader(&mut self, gl: &gl::Gl) {
        const VS_SRC: &[u8] = b"
#version 100
precision mediump float;

uniform mat4 matMV;
uniform mat4 matPrj;

attribute vec3 position;

void main() {
//    gl_Position = vec4(position, 1.0);
    gl_Position = matPrj * matMV * vec4(position, 1.0);
}
\0";

        const FS_SRC: &[u8] = b"
#version 100
precision mediump float;

uniform vec3 color;

void main() {
    gl_FragColor = vec4(color, 1.0);
    // gl_FragColor = vec4(0., 1., 0., 1.0);
}
\0";
        unsafe {
            use crate::utility::{get_location, compile_shaders};
            self.program = compile_shaders(gl, VS_SRC, FS_SRC);
            self.loc_mat_modelview = get_location(gl, "matMV", self.program);
            self.loc_mat_projection = get_location(gl, "matPrj", self.program);
            self.loc_color = get_location(gl, "color", self.program);
        }
    }

    pub fn initialize(
        &mut self,
        gl: &gl::Gl,
        vtx2xyz: &Vec<f32>) {
        unsafe {
            let mut vb = std::mem::zeroed();
            gl.GenBuffers(1, &mut vb);
            gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx2xyz.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx2xyz.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            if gl.BindVertexArray.is_loaded() {
                self.vao = std::mem::zeroed();
                gl.GenVertexArrays(1, &mut self.vao);
                gl.BindVertexArray(self.vao);
            }

            let pos_attrib = gl.GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                3 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
        }


    }

    pub fn draw_frame(
        &self,
        gl: &gl::Gl,
        mat_modelview: &[f32],
        mat_projection: &[f32]) {
        let mp0 = mat_projection;
        let mp1: [f32;16] = [ // mp1 = [z flip] * mp0
            mp0[0], mp0[1], -mp0[2], mp0[3],
            mp0[4], mp0[5], -mp0[6], mp0[7],
            mp0[8], mp0[9], -mp0[10], mp0[11],
            mp0[12], mp0[13], -mp0[14], mp0[15] ];
        unsafe {
            gl.UseProgram(self.program);
            gl.BindVertexArray(self.vao);
            gl.Uniform3f(self.loc_color, self.color[0], self.color[1], self.color[2]);
            gl.UniformMatrix4fv(self.loc_mat_modelview, 1, gl::FALSE, mat_modelview.as_ptr());
            gl.UniformMatrix4fv(self.loc_mat_projection, 1, gl::FALSE, mp1.as_ptr());
            gl.DrawArrays(self.mode, 0, self.elem_size.try_into().unwrap());
        }
    }
}
