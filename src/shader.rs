use std::io::Read;
use std::path::Path;
use std::fs::File;
use std::ffi::{CString};

pub struct UnlinkedProgram {
    shaders: Vec<gl::types::GLuint>,
}

impl UnlinkedProgram {
    pub fn new() -> Self {
        Self {
            shaders: Vec::new()
        }
    }

    pub fn add_shader(&mut self, source_path: &Path, kind: gl::types::GLuint) -> Result<(), String> {
        let mut file = File::open(source_path).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        
        let source = CString::new(source).unwrap();

        // WARNING: we can't trust `kind` is right
        let id = unsafe { gl::CreateShader(kind) };

        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error: CString = unsafe { CString::from_vec_unchecked(vec![b' '; len as usize]) };

            unsafe {
                gl::GetShaderInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(())
    }

    pub fn link(self) -> Result<Program, String> {
        let program = Program { id: unsafe { gl::CreateProgram() } };

        for shader in &self.shaders {
            unsafe { gl::AttachShader(program.id, *shader); }
        }

        unsafe { gl::LinkProgram(program.id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error: CString = unsafe { CString::from_vec_unchecked(vec![b' '; len as usize]) };

            unsafe {
                gl::GetProgramInfoLog(
                    program.id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }
        
        for shader in &self.shaders {
            unsafe { gl::DetachShader(program.id, *shader); }
        }
        
        Ok(program)
    }
}

impl Drop for UnlinkedProgram {
    fn drop(&mut self) {
        for shader in &self.shaders {
            unsafe {
                gl::DeleteShader(*shader);
            }
        }
    }
}

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn deactivate() {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
