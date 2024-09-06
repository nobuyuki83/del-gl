//! draw array of position and color

use crate::gl;

pub struct Drawer {
    pub program: gl::types::GLuint,
    pub mode: gl::types::GLenum,
}

impl Drawer {
    pub fn compile_shader(&mut self, gl: &gl::Gl) {
        const VS_SRC: &[u8] = b"
#version 100
precision mediump float;

attribute vec2 position;
attribute vec3 color;

varying vec3 v_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

        const FS_SRC: &[u8] = b"
#version 100
precision mediump float;

varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
\0";
        unsafe {
            self.program = crate::utility::compile_shaders(gl, VS_SRC, FS_SRC);
        }
    }

    pub fn initialize(&self, gl: &gl::Gl, vtx2xyrgb: &Vec<f32>) {
        let mut vb = 0_u32;
        unsafe {
            gl.GenBuffers(1, &mut vb);
            gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx2xyrgb.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx2xyrgb.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            if gl.BindVertexArray.is_loaded() {
                let mut vao = std::mem::zeroed();
                gl.GenVertexArrays(1, &mut vao);
                gl.BindVertexArray(vao);
            }

            let pos_attrib = gl.GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl.GetAttribLocation(self.program, b"color\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
        }
    }

    pub fn draw_frame(&self, gl: &gl::Gl) {
        unsafe {
            gl.UseProgram(self.program);
            gl.DrawArrays(self.mode, 0, 3);
        }
    }
}
