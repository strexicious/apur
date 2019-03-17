use std::fs;
use std::char;
use std::path::Path;
use std::ffi::{CString};
use std::collections::HashMap;

enum UniformType {
    Vec3Uniform,
}

pub struct Uniform {
    loc: gl::types::GLint,
    uniform_type: UniformType,
}

impl Uniform {
    fn new(loc: gl::types::GLint, program_id: gl::types::GLuint) -> Result<Self, String> {
        let mut uniform_type = gl::INVALID_ENUM;
        let mut _uniform_size: gl::types::GLint = 0; // TODO: check if this is actually needed,
        // or can we trick it into not writing this at all
        unsafe {
            gl::GetActiveUniform(
                program_id,
                loc as gl::types::GLuint, // note, loc is prequisted to be positive
                0,
                std::ptr::null_mut(),
                &mut _uniform_size as *mut gl::types::GLint,
                &mut uniform_type as *mut gl::types::GLenum,
                std::ptr::null_mut()
            );
        }

        match uniform_type {
            gl::FLOAT_VEC3 => Ok(Self { loc, uniform_type: UniformType::Vec3Uniform }),
            _ => Err(String::from("Uniform type not supported")),
        }
    }

    fn load_data(&self, data: &[f32]) -> Result<(), String> {
        use UniformType::*;
        
        match self.uniform_type {
            Vec3Uniform => {
                if data.len() != 3 {
                    return Err(String::from("`vec3` uniform requires 3 components"));
                }
                unsafe { gl::Uniform3fv(self.loc, 1, data.as_ptr()); }
            }
        }
        Ok(())
    }
}

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

            // reserve buffer
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
        let program = Program { id: UnlinkedProgram::create_program()?, uniforms: HashMap::new() };

        for shader in &self.shaders {
            unsafe { gl::AttachShader(program.id, *shader); }
        }

        unsafe { gl::LinkProgram(program.id); }

        // I don't think if it fails it detaches automatically, so we do it always
        for shader in &self.shaders {
            unsafe { gl::DetachShader(program.id, *shader); }
        }

        UnlinkedProgram::handle_link_error(program.id)?;
        
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
    uniforms: HashMap<String, Uniform>,
}

impl Program {
    pub fn add_uniform(&mut self, name: String) -> Result<(), String> {
        if self.uniforms.get(&name).is_some() {
            return Err(String::from("Uniform already added"));
        }
        
        if name.find(char::is_whitespace).is_some() {
            return Err(String::from("The argument `name` cannot contain a whitespace"));
        }
        
        let c_name = CString::new(name.clone()).unwrap();
        let loc = unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr()) };
        if loc == -1 {
            return Err(String::from("Uniform not found"));
        }

        self.uniforms.insert(name, Uniform::new(loc, self.id)?);
        Ok(())
    }

    pub fn load_uniform(&self, name: &str, data: &[f32]) -> Result<(), String> {
        self.activate();
        self.uniforms[name].load_data(data)?;
        Program::deactivate();
        Ok(())
    }
    
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
