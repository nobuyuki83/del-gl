use crate::gl::types::GLsizei;
use std::ffi::CStr;

pub mod view_ui_state;

/*
pub mod gl {
    #![allow(
    clippy::manual_non_exhaustive,
    clippy::too_many_arguments,
    clippy::unused_unit,
    clippy::upper_case_acronyms,
    non_camel_case_types
    )]

    pub use self::Gles2 as Gl;

    // gl_bindings.rs is generated in build.rs using https://crates.io/crates/gl_generator
    // include!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/gl_bindings.rs"));
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
 */

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
    pub use Gles2 as Gl;
}

pub unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    gl.ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl.CompileShader(shader);
    {
        // show error message if fails
        let mut success: gl::types::GLint = 0;
        gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != 1 {
            let mut info_log = [0i8; 512];
            let mut size: GLsizei = 0;
            gl.GetShaderInfoLog(shader, 512, &mut size, info_log.as_mut_ptr());
            let info_log: Vec<u8> = info_log
                .iter()
                .take_while(|&v| *v != 0)
                .map(|&v| v as u8)
                .collect();
            let a = std::str::from_utf8(&info_log).unwrap();
            println!("Shader Compile Error! --> {}", a);
            panic!();
        }
    }
    shader
}

pub fn set_shader_program(
    gl: &gl::Gl,
    vertex_shader_source: &[u8],
    fragment_shader_source: &[u8],
) -> gl::types::GLuint {
    if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
        println!("Running on {}", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(&gl, gl::VERSION) {
        println!("OpenGL Version {}", version.to_string_lossy());
    }

    if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
        println!("Shaders version on {}", shaders_version.to_string_lossy());
    }
    unsafe {
        let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, vertex_shader_source);
        let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, fragment_shader_source);
        let program = gl.CreateProgram();
        gl.AttachShader(program, vertex_shader);
        gl.AttachShader(program, fragment_shader);
        gl.LinkProgram(program);
        /*
        {
            // this will always fail even though the shader programs successfully links
            let mut success: gl::types::GLint = 0;
            gl.GetShaderiv(program, gl::LINK_STATUS, &mut success);
            dbg!(success);
            if success == 0 {
                let mut infoLog = [0i8;512];
                let mut size : GLsizei = 0;
                gl.GetShaderInfoLog(program, 512, &mut size, infoLog.as_mut_ptr());
                let infoLog: Vec<u8> = infoLog.iter().take_while(|&v| *v != 0 )
                    .map(|&v| v as u8 ).collect();
                let a = std::str::from_utf8(&infoLog).unwrap();
                println!("Shader Link Error! --> {}", a);
                panic!();
            }
        }
         */
        gl.UseProgram(program);
        gl.DeleteShader(vertex_shader);
        gl.DeleteShader(fragment_shader);
        program
    }
}

pub fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

pub mod app_internal;
pub mod array;
pub mod array_vtxcolor;
pub mod mesh;
pub mod mesh_colormap;
pub mod mesh_tex;
pub mod utility;

// folder
// pub mod glutin;
pub mod nalgebra;
