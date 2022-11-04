use crate::gl;

pub unsafe fn compile_shaders(
    gl: &gl::Gl,
    src_vertex: &[u8],
    src_fragment: &[u8]) -> gl::types::GLuint {
    let vs = gl.CreateShader(gl::VERTEX_SHADER);
    gl.ShaderSource(vs, 1, [src_vertex.as_ptr() as *const _].as_ptr(), std::ptr::null());
    gl.CompileShader(vs);

    let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
    gl.ShaderSource(fs, 1, [src_fragment.as_ptr() as *const _].as_ptr(), std::ptr::null());
    gl.CompileShader(fs);

    let id_program = gl.CreateProgram();
    gl.AttachShader(id_program, vs);
    gl.AttachShader(id_program, fs);
    gl.LinkProgram(id_program);
    assert!( gl.IsProgram(id_program) != 0 );
    {
        let mut success: gl::types::GLint = 0;
        gl.GetProgramiv(id_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let info_log: [i8; 512] = [0; 512];
            let mut length: i32 = 512;
            gl.GetProgramInfoLog(id_program, 512, &mut length, info_log.as_ptr() as *mut _);
            println!("{}", length);
            let info_log0 = String::from_utf8(info_log.iter().map(|&c| c as u8).collect());
            println!("ERROR::SHADER::PROGRAM::LINKING_FAILED {:?}", info_log0);
        }
    }
    gl.DeleteShader(vs);
    gl.DeleteShader(fs);
    id_program
}