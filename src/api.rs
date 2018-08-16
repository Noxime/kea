extern crate gl;
extern crate glutin;

use self::gl::types::*;
use std::ffi::CString;
use std::ptr;

#[derive(Debug)]
pub struct Shader(u32);
#[derive(Debug)]
pub enum ShaderError {
    StringError,
    CompileError(String),
}

pub trait GfxApi {
    // clear the current buffer with specified color
    fn clear(&self, r: f32, g: f32, b: f32);
    // take shader sources and compile them down to a shader
    fn compile_shader(&self, vertex: &str, fragment: &str) -> Result<Shader, ShaderError>;
}

pub struct GLApi;
impl GfxApi for GLApi {
    fn clear(&self, r: f32, g: f32, b: f32) {
        unsafe {
            gl::ClearColor(r, g, b, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn compile_shader(&self, vertex: &str, fragment: &str) -> Result<Shader, ShaderError> {
        log!("Compiling new shader");
        let vs = self.compile_one(vertex, gl::VERTEX_SHADER)?;
        log!("Vertex shader done");
        let fs = self.compile_one(fragment, gl::FRAGMENT_SHADER)?;
        log!("Fragment shader done");

        Ok(Shader(unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            log!("Shader linked");

            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            log!("Shader link status: {}", status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                log!("Error log length: {}", len);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                gl::DeleteProgram(program);

                return Err(ShaderError::CompileError(
                    String::from_utf8(buf).map_err(|_| ShaderError::StringError)?,
                ));
            }

            log!("Shader created with internal id: {}", program);
            program
        }))
    }
}

impl GLApi {
    fn compile_one(&self, source: &str, type_: GLenum) -> Result<GLuint, ShaderError> {
        Ok(unsafe {
            let shader = gl::CreateShader(type_);
            gl::ShaderSource(
                shader,
                1,
                &CString::new(source)
                    .map_err(|_| ShaderError::StringError)?
                    .as_ptr(),
                ptr::null(),
            );
            gl::CompileShader(shader);

            // Get the compile status
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            log!("Shader compile status: {}", status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                // don't leak
                gl::DeleteShader(shader);

                return Err(ShaderError::CompileError(
                    String::from_utf8(buf).map_err(|_| ShaderError::StringError)?,
                ));
            }
            shader
        })
    }
}
