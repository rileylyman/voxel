use glfw::{Action, Context, Key};
use std::convert::TryInto;

extern "system" fn gl_debug_callback(
    _source: u32,
    _type: u32,
    _id: u32,
    _sev: u32,
    _length: i32,
    msg: *const i8,
    _data: *mut std::ffi::c_void,
) -> () {
    println!("OpenGL errored: {:?}", unsafe {
        std::ffi::CString::from_raw(std::mem::transmute(msg))
    });
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 1));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let (mut window, events) = glfw
        .create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_resizable(true);
    window.set_key_polling(true);
    window.set_size_polling(true);

    gl::load_with(|s| window.get_proc_address(s));

    let program;
    unsafe {
        let vshader = gl::CreateShader(gl::VERTEX_SHADER);
        let vert_source = std::ffi::CString::new(VERTEX_SOURCE).unwrap();
        gl::ShaderSource(
            vshader,
            1,
            std::mem::transmute(&vert_source),
            std::ptr::null(),
        );
        gl::CompileShader(vshader);

        let mut vertex_compiled = 0;
        gl::GetShaderiv(vshader, gl::COMPILE_STATUS, &mut vertex_compiled);

        if vertex_compiled != gl::TRUE.into() {
            let mut log_length = 0;
            let mut message: [gl::types::GLchar; 1024] = [0; 1024];
            gl::GetShaderInfoLog(vshader, 1024, &mut log_length, message.as_mut_ptr());
            panic!(
                "Vertex shader error: {:?}",
                std::ffi::CString::from_raw(message.as_mut_ptr())
            );
        }

        let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let frag_source = std::ffi::CString::new(FRAG_SOURCE).unwrap();
        gl::ShaderSource(
            fshader,
            1,
            std::mem::transmute(&frag_source),
            std::ptr::null(),
        );
        gl::CompileShader(fshader);

        let mut frag_compiled = 0;
        gl::GetShaderiv(fshader, gl::COMPILE_STATUS, &mut frag_compiled);

        if frag_compiled != gl::TRUE.into() {
            let mut log_length = 0;
            let mut message: [gl::types::GLchar; 1024] = [0; 1024];
            gl::GetShaderInfoLog(fshader, 1024, &mut log_length, message.as_mut_ptr());
            println!(
                "Fragment shader error: {:?}",
                std::ffi::CString::from_raw(message.as_mut_ptr())
            );
        }

        program = gl::CreateProgram();
        gl::AttachShader(program, vshader);
        gl::AttachShader(program, fshader);
        gl::LinkProgram(program);

        let mut program_linked = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut program_linked);
        if program_linked != gl::TRUE.into() {
            let mut log_length = 0;
            let mut message: [gl::types::GLchar; 1024] = [0; 1024];
            gl::GetProgramInfoLog(fshader, 1024, &mut log_length, message.as_mut_ptr());
            println!(
                "Program link error: {:?}",
                std::ffi::CString::from_raw(message.as_mut_ptr())
            );
        }

        gl::DeleteShader(vshader);
        gl::DeleteShader(fshader);
    }

    let triangle = vec![0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let indices = vec![0u32, 1, 2];

    let mut vbo: gl::types::GLuint = 0;
    let mut ebo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (triangle.len() * std::mem::size_of::<f32>()) as isize,
            std::mem::transmute(triangle.as_ptr()),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (triangle.len() * std::mem::size_of::<u32>())
                .try_into()
                .unwrap(),
            std::mem::transmute(indices.as_ptr()),
            gl::STATIC_DRAW,
        );
    }

    unsafe {
        gl::Disable(gl::CULL_FACE);
    }

    let (width, height) = window.get_framebuffer_size();
    println!("{}, {}", width, height);
    unsafe {
        gl::Viewport(0, 0, width, height);
        gl::DebugMessageCallback(Some(gl_debug_callback), std::ptr::null());
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DONT_CARE,
            0,
            std::ptr::null(),
            gl::TRUE,
        );
    }

    unsafe {
        gl::ClearColor(0.5, 0.3, 0.7, 1.0);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindVertexArray(vao);
            gl::UseProgram(program);
            gl::ClearColor(0.5, 0.3, 0.7, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Size(new_x, new_y) => {
            // window.set_size(new_x, new_y);
            println!("Resized to {} and {}", new_x, new_y);
            unsafe {
                gl::Viewport(0, 0, new_x, new_y);
            }
        }
        _ => {}
    }
}

const VERTEX_SOURCE: &'static str = "
#version 330 core
layout (location = 0) in vec3 a_pos;
void main() {
  gl_Position = vec4(a_pos.x, a_pos.y, a_pos.z, 1.0);
}
";

const FRAG_SOURCE: &'static str = "
#version 330 core

out vec4 FragColor;

void main() {
  FragColor = vec4(0.8, 0.0, 0.1, 1.0);
}
";