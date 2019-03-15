use std::fs;
use std::path::Path;
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

    fn create_shader(shader_type: gl::types::GLuint) -> Result<gl::types::GLuint, String> {
        match shader_type {
            gl::VERTEX_SHADER | gl::GEOMETRY_SHADER | gl::FRAGMENT_SHADER => {
                let id = unsafe { gl::CreateShader(shader_type) };
                if id == 0 {
                    return Err(String::from("[ GL ] Failed to create a shader"));
                }
                Ok(id)
            },
            _ => Err(String::from("Invalid shader type"))
        }        
    }

    fn handle_compile_error(id: gl::types::GLuint) -> Result<(), String> {
        let mut success: gl::types::GLint = 1;
        unsafe { gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success); }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe { gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len); }

            // reserve buffer
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

    pub fn add_shader(
        &mut self, source_path: &Path,
        shader_type: gl::types::GLuint
    ) -> Result<(), String> {
        let source = fs::read_to_string(source_path)
            .map_err(move |_error| {
                String::from("[ OS ] Failed to open/read the shader file")
            })?;

        let source = CString::new(source).unwrap();

        let id = UnlinkedProgram::create_shader(shader_type)?;

        // try to compile
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        // handle errors
        UnlinkedProgram::handle_compile_error(id)?;
        
        self.shaders.push(id);

        Ok(())
    }

    fn handle_link_error(id: gl::types::GLuint) -> Result<(), String> {
        let mut success: gl::types::GLint = 1;
        unsafe { gl::GetProgramiv(id, gl::LINK_STATUS, &mut success); }
        
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe { gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len); }

            let error: CString = unsafe { CString::from_vec_unchecked(vec![b' '; len as usize]) };
            unsafe {
                gl::GetProgramInfoLog(
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

    fn create_program() -> Result<gl::types::GLuint, String> {
        let id = unsafe { gl::CreateProgram() };
        if id == 0 {
            return Err(String::from("[ GL ] Failed to create a program"));
        }
        Ok(id)
    }

    pub fn link(self) -> Result<Program, String> {
        let program = Program { id: UnlinkedProgram::create_program()? };

        for shader in &self.shaders {
            unsafe { gl::AttachShader(program.id, *shader); }
        }

        unsafe { gl::LinkProgram(program.id); }

        UnlinkedProgram::handle_link_error(program.id)?;
        
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
