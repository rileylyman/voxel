use glfw::{Action, Context, Key};
use std::convert::TryInto;

enum CameraPosition {
    SphericalAbout {
        origin: glam::Vec3,
        radius: f32,
        theta: f32,
        phi: f32,
    },
    Absolute {
        position: glam::Vec3,
        look_at: glam::Vec3,
    },
}

struct Camera {
    pos: CameraPosition,
}

fn pos_from_theta_phi(theta: f32, phi: f32) -> glam::Vec3 {
    glam::Vec3::new(theta.cos() * phi.cos(), phi.sin(), theta.sin() * phi.cos())
}

impl Camera {
    pub fn get_view(&self) -> glam::Mat4 {
        match self.pos {
            CameraPosition::Absolute { position, look_at } => {
                glam::Mat4::look_at_rh(position, look_at, glam::Vec3::new(0.0, 1.0, 0.0))
            }
            CameraPosition::SphericalAbout {
                origin,
                radius,
                theta,
                phi,
            } => {
                let pos = radius * pos_from_theta_phi(theta, phi);
                let pos = origin + pos;
                glam::Mat4::look_at_rh(
                    pos,
                    -(pos - origin).normalize(),
                    glam::Vec3::new(0.0, 1.0, 0.0),
                )
            }
        }
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 5));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let (mut window, events) = glfw
        .create_window(
            1600,
            1080,
            "Hello this is window",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_resizable(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_key_polling(true);
    window.set_size_polling(true);
    window.set_scroll_polling(true);

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

    let vertices = vec![
        -0.5f32, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, // bottom right
        0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, // top right
        -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 1.0, // top let
        // right ace
        0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, // bottom let
        0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 1.0, 0.0, // bottom right
        0.5, 0.5, -0.5, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 1.0, // top let
        // let ace
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 0.0, // bottom let
        -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, 1.0, 0.0, // bottom right
        -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0, 1.0, // top right
        -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0, // top let
        // bottom ace
        -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 0.0, // bottom let
        0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 1.0, 0.0, // bottom right
        0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 1.0, // top right
        -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.0, 1.0, // top let
        // top ace
        -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, // bottom let
        0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, // top right
        -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, // top let
        // back ace
        -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom let
        0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 0.0, // bottom right
        0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, // top right
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 1.0, // top left
        // bottom plane
        -10.5, -1.5, -10.5, 0.0, 1.0, 0.0, 0.0, 0.0, // bottom let
        10.5, -1.5, -10.5, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        10.5, -1.5, 10.5, 0.0, 1.0, 0.0, 1.0, 1.0, // top right
        -10.5, -1.5, 10.5, 0.0, 1.0, 0.0, 0.0, 1.0, // top let
    ];
    let indices = vec![
        0u32, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17,
        18, 18, 19, 16, 20, 21, 22, 22, 23, 20, 24, 25, 26, 26, 27, 24,
    ];

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
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            std::mem::transmute(vertices.as_ptr()),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * std::mem::size_of::<f32>()) as i32,
            (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u32>())
                .try_into()
                .unwrap(),
            std::mem::transmute(indices.as_ptr()),
            gl::STATIC_DRAW,
        );
    }

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
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
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            let view_matrix = CAMERA.get_view();

            let (width, height) = window.get_framebuffer_size();
            let aspect_ratio = width as f32 / height as f32;
            let proj_matrix = glam::Mat4::perspective_rh_gl(
                90.0f32.to_radians() / aspect_ratio,
                aspect_ratio,
                0.1,
                1000.0,
            );

            let view_name = std::ffi::CString::new("u_view").unwrap();
            let loc = gl::GetUniformLocation(program, view_name.as_ptr() as *const i8);
            gl::ProgramUniformMatrix4fv(
                program,
                loc,
                1,
                gl::FALSE,
                view_matrix.to_cols_array().as_ptr(),
            );

            let proj_name = std::ffi::CString::new("u_proj").unwrap();
            let other_loc = gl::GetUniformLocation(program, proj_name.as_ptr() as *const i8);
            gl::ProgramUniformMatrix4fv(
                program,
                other_loc,
                1,
                gl::FALSE,
                proj_matrix.to_cols_array().as_ptr(),
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindVertexArray(vao);
            gl::UseProgram(program);
            //gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            // gl::DrawArrays(gl::TRIANGLES, 0, 6 * 6);
            gl::DrawElements(gl::TRIANGLES, 36 + 6, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.swap_buffers();
    }
}

static mut MOUSE_X_POS: f64 = 0.0;
static mut MOUSE_Y_POS: f64 = 0.0;
static mut _CAMERA: Camera = Camera {
    pos: CameraPosition::Absolute {
        position: glam::Vec3::Z,
        look_at: glam::Vec3::ZERO,
    },
};
static mut CAMERA: Camera = Camera {
    pos: CameraPosition::SphericalAbout {
        origin: glam::Vec3::ZERO,
        radius: 3.0,
        theta: 3.14 / 2.0,
        phi: 0.0,
    },
};
static mut IS_PANNING: bool = false;
static mut ORIGINAL_X: f64 = 0.0;
static mut ORIGINAL_Y: f64 = 0.0;

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
        glfw::WindowEvent::MouseButton(glfw::MouseButton::Button3, glfw::Action::Press, _) => unsafe {
            IS_PANNING = true;
        },
        glfw::WindowEvent::MouseButton(glfw::MouseButton::Button3, glfw::Action::Release, _) => unsafe {
            IS_PANNING = false;
        },
        glfw::WindowEvent::Scroll(_, amount) => unsafe {
            match CAMERA.pos {
                CameraPosition::SphericalAbout {
                    origin: _,
                    ref mut radius,
                    theta: _,
                    phi: _,
                } => {
                    *radius -= (amount as f32) * 0.1;
                }
                _ => {}
            }
        },
        glfw::WindowEvent::CursorPos(x, y) => unsafe {
            if IS_PANNING {
                match CAMERA.pos {
                    CameraPosition::Absolute {
                        ref mut position,
                        ref mut look_at,
                    } => {
                        *position += glam::Vec3::new(
                            -(x - MOUSE_X_POS) as f32,
                            (y - MOUSE_Y_POS) as f32,
                            0.0,
                        ) * 0.002;
                        *look_at += glam::Vec3::new(
                            -(x - MOUSE_X_POS) as f32,
                            (y - MOUSE_Y_POS) as f32,
                            0.0,
                        ) * 0.002;
                    }
                    CameraPosition::SphericalAbout {
                        origin: ref mut _origin,
                        radius: ref mut _radius,
                        ref mut phi,
                        ref mut theta,
                    } => {
                        // println!("theta={}, phi={}", theta.to_degrees(), phi.to_degrees());
                        if (x - ORIGINAL_X).abs() > (y - ORIGINAL_Y).abs() {
                            *theta += (x - MOUSE_X_POS) as f32 * 0.002;
                        } else {
                            *phi += (y - MOUSE_Y_POS) as f32 * 0.008;
                            *phi = phi.clamp(-3.14 / 2.0, 3.14 / 2.0);
                        }
                    }
                }
            } else {
                ORIGINAL_X = x;
                ORIGINAL_Y = y;
            }
            MOUSE_X_POS = x;
            MOUSE_Y_POS = y;
        },
        _ => {}
    }
}

const VERTEX_SOURCE: &'static str = "
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;

uniform mat4 u_view;
uniform mat4 u_proj;

out vec3 io_position;
out vec3 io_light_pos;
out vec3 io_normal;

void main() {
  io_light_pos = vec3(u_view * vec4(0.0, 2.0, 1.0, 1.0));
  io_position = vec3(u_view * vec4(a_pos, 1.0));
  io_normal = vec3(u_view * vec4(a_normal, 0.0));

  gl_Position = u_proj * u_view * vec4(a_pos, 1.0);
}
";

const FRAG_SOURCE: &'static str = "
#version 330 core

out vec4 FragColor;

vec4 k_light_color = vec4(1.0, 1.0, 1.0, 1.0);
// vec4 k_object_color = vec4(0.8, 0.2, 0.2, 1.0);
vec4 k_object_color = vec4(0.5, 0.5, 0.5, 1.0);

float k_ambient_coefficient = 0.3;
float k_diffuse_coefficient = 0.3;
float k_specular_coefficient = 0.3;
float k_p = 16;

in vec3 io_position;
in vec3 io_light_pos;
in vec3 io_normal;

void main() {

    vec3 to_light = normalize(io_light_pos - io_position);
    vec3 to_camera = normalize(-io_position);
    vec3 halfway = normalize(to_camera + to_light);
    float dist2 = dot(to_light, to_light);

    vec4 ambient_component = k_ambient_coefficient * k_object_color;

    vec4 diffuse_component = k_diffuse_coefficient * (k_light_color / dist2) * max(0, dot(io_normal, to_light));

    vec4 specular_component = k_specular_coefficient * (k_light_color / dist2) * pow(max(0, dot(io_normal, halfway)), k_p);

    FragColor = ambient_component + diffuse_component + specular_component;
}
";

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
