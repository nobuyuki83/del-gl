//! draw mesh position. The RGB color is defined par index

use crate::gl;

struct ElementBufferObject {
    mode: gl::types::GLenum,
    elem_size: usize,
    ebo: gl::types::GLuint,
    color: Option<[f32; 3]>,
}

pub struct Drawer {
    program: gl::types::GLuint,
    pub ndim: i32,
    num_point: i32,
    vao: gl::types::GLuint,
    // uniform variables
    loc_texture: gl::types::GLint,
    loc_color: gl::types::GLint,
    loc_is_texture: gl::types::GLint,
    loc_mat_modelview: gl::types::GLint,
    loc_mat_projection: gl::types::GLint,
    // elemenb buffer object
    ebos: Vec<ElementBufferObject>,
}

impl Drawer {
    pub fn new() -> Self {
        Drawer {
            program: 0,
            ndim: 0,
            num_point: 0,
            vao: 0,
            loc_texture: -1,    // -1 is the failure flag
            loc_color: -1,      // -1 is the failure flag
            loc_is_texture: -1, // -1 is the failure flag
            loc_mat_modelview: -1,
            loc_mat_projection: -1,
            ebos: Vec::<ElementBufferObject>::new(),
        }
    }

    pub fn compile_shader(&mut self, gl: &gl::Gl) {
        const VS_SRC: &[u8] = b"
#version 330

uniform mat4 matMV;
uniform mat4 matPrj;

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texIn;
out vec2 texPrj;

void main() {
    gl_Position = matPrj * matMV * vec4(position, 1.0);
    texPrj = texIn;
    // gl_Position = vec4(position, 1.0);
}
\0";

        const FS_SRC: &[u8] = b"
#version 330

uniform sampler2D myTextureSampler;
uniform vec3 color;
uniform bool is_texture;

in vec2 texPrj;
out vec4 FragColor;

void main() {
    if( is_texture ){
        FragColor = texture(myTextureSampler,texPrj);
    }
    else {
        FragColor = vec4(color, 1.0);
    }
}
\0";
        use crate::utility::{compile_shaders, get_uniform_location};
        unsafe {
            self.program = compile_shaders(gl, VS_SRC, FS_SRC);
            self.loc_mat_modelview = get_uniform_location(gl, "matMV", self.program);
            self.loc_mat_projection = get_uniform_location(gl, "matPrj", self.program);
            self.loc_texture = get_uniform_location(gl, "myTextureSampler", self.program);
            self.loc_color = get_uniform_location(gl, "color", self.program);
            self.loc_is_texture = get_uniform_location(gl, "is_texture", self.program);

            if gl.BindVertexArray.is_loaded() {
                let mut vao0 = std::mem::zeroed();
                gl.GenVertexArrays(1, &mut vao0);
                self.vao = vao0;
                gl.BindVertexArray(self.vao);
            }
        }
    }

    pub fn add_element<T>(
        &mut self,
        gl: &gl::Gl,
        mode: gl::types::GLenum,
        elem2vtx: &Vec<T>,
        color: Option<[f32; 3]>,
    ) where
        T: 'static + Copy + num_traits::AsPrimitive<gl::types::GLuint>,
    {
        use crate::gl::types::GLuint;
        unsafe {
            gl.BindVertexArray(self.vao);
            let mut ebo0 = std::mem::zeroed();
            gl.GenBuffers(1, &mut ebo0);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo0);
            let elem_vtx0: Vec<GLuint> = elem2vtx.iter().map(|i| (*i).as_()).collect();
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (elem_vtx0.len() * std::mem::size_of::<usize>()) as gl::types::GLsizeiptr,
                elem_vtx0.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            self.ebos.push(ElementBufferObject {
                mode,
                elem_size: elem_vtx0.len(),
                ebo: ebo0,
                color,
            });
        }
    }

    pub fn update_vertex(&mut self, gl: &gl::Gl, vtx2xyz: &Vec<f32>, ndim: i32) {
        self.ndim = ndim;
        self.num_point = vtx2xyz.len() as i32 / self.ndim;
        unsafe {
            gl.BindVertexArray(self.vao);
            //
            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx2xyz.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx2xyz.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl.GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                self.ndim,
                gl::FLOAT,
                0,
                self.ndim * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
        }
    }

    pub fn set_texture_uv(&mut self, gl: &gl::Gl, vtx2tex: &Vec<f32>) {
        unsafe {
            gl.BindVertexArray(self.vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx2tex.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx2tex.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            let pos_attrib = gl.GetAttribLocation(self.program, b"texIn\0".as_ptr() as *const _);
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            ); // gl24
        }
    }

    pub fn draw(&self, gl: &gl::Gl, mat_modelview: &[f32], mat_projection: &[f32]) {
        let mp0 = mat_projection;
        let mp1: [f32; 16] = [
            // mp1 = [z flip] * mp0
            mp0[0], mp0[1], -mp0[2], mp0[3], mp0[4], mp0[5], -mp0[6], mp0[7], mp0[8], mp0[9],
            -mp0[10], mp0[11], mp0[12], mp0[13], -mp0[14], mp0[15],
        ];
        unsafe {
            gl.UseProgram(self.program);
            gl.BindVertexArray(self.vao);
            for ebo in &self.ebos {
                match ebo.color {
                    Some(color) => {
                        gl.Uniform1i(self.loc_is_texture, 0);
                        gl.Uniform3f(self.loc_color, color[0], color[1], color[2]);
                    }
                    _ => {
                        gl.Uniform1i(self.loc_is_texture, 1);
                    }
                }
                gl.UniformMatrix4fv(self.loc_mat_modelview, 1, gl::FALSE, mat_modelview.as_ptr());
                gl.UniformMatrix4fv(self.loc_mat_projection, 1, gl::FALSE, mp1.as_ptr());
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

    pub fn draw_points(&self, gl: &gl::Gl, mat_modelview: &[f32], mat_projection: &[f32]) {
        let mp0 = mat_projection;
        let mp1: [f32; 16] = [
            // mp1 = [z flip] * mp0
            mp0[0], mp0[1], -mp0[2], mp0[3], mp0[4], mp0[5], -mp0[6], mp0[7], mp0[8], mp0[9],
            -mp0[10], mp0[11], mp0[12], mp0[13], -mp0[14], mp0[15],
        ];
        unsafe {
            gl.UseProgram(self.program);
            gl.BindVertexArray(self.vao);
            gl.UniformMatrix4fv(self.loc_mat_modelview, 1, gl::FALSE, mat_modelview.as_ptr());
            gl.UniformMatrix4fv(self.loc_mat_projection, 1, gl::FALSE, mp1.as_ptr());
            gl.DrawArrays(gl::POINTS, 0, self.num_point);
        }
    }
}
