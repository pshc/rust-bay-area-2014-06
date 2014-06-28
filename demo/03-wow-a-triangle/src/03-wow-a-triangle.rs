// Copyright 2014 Brendan Zabarauskas.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate cgmath;
extern crate gl;
extern crate glfw;
extern crate native;

use gl::types::{GLchar, GLenum, GLfloat};
use gl::types::{GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};
use glfw::Context;
use cgmath::angle::rad;
use cgmath::array::Array2;
use cgmath::matrix::{ToMatrix4};
use cgmath::quaternion::Quaternion;
use cgmath::rotation::Rotation3;
use std::mem;
use std::ptr;

static VERTEX_DATA: [GLfloat, ..18] = [
     0.0,  0.5,    0.0,  0.0,  1.0,  1.0,
     0.5, -0.5,    0.0,  1.0,  0.0,  1.0,
    -0.5, -0.5,    1.0,  0.0,  0.0,  1.0,
];

static VERTEX_SHADER_SRC: &'static [u8] = b"
    #version 150
    uniform mat4 modelview;
    in vec2 position;
    in vec4 color;
    out vec4 in_color;
    void main() {
       gl_Position = modelview * vec4(position, 0.0, 1.0);
       in_color = color;
    }
";

static FRAGMENT_SHADER_SRC: &'static [u8] = b"
    #version 150
    in vec4 in_color;
    out vec4 out_color;
    void main() {
       out_color = in_color;
    }
";

fn compile_shader(src: &[u8], ty: GLenum) -> GLuint {
    let shader = gl::CreateShader(ty);
    let len = src.len() as GLint;
    unsafe { gl::ShaderSource(shader, 1, &(src.as_ptr() as *GLchar), &len) };
    gl::CompileShader(shader);
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    program
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    // initialise context (handle can't be moved between threads)
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Choose a GL profile that is compatible with OS X 10.7+
    glfw.window_hint(glfw::ContextVersion(3, 2));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

    let (window, events) = glfw.create_window(800, 600, "Spiiiin", glfw::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);

    // It is essential to make the context current before calling `gl::load_with`.
    window.make_current();

    // Load the OpenGL function pointers
    gl::load_with(|s| glfw.get_proc_address(s));

    // Create GLSL shaders
    let vs = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;

    // Create Vertex Array Object
    unsafe { gl::GenVertexArrays(1, &mut vao) };
    gl::BindVertexArray(vao);

    let sizeof_float = mem::size_of::<GLfloat>();

    // Create a Vertex Buffer Object and copy the vertex data to it
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (VERTEX_DATA.len() * sizeof_float) as GLsizeiptr,
                       VERTEX_DATA.as_ptr() as *GLvoid,
                       gl::STATIC_DRAW);
    }

    // Use the shader program
    gl::UseProgram(program);

    // Attributes
    unsafe {
        "out_color".with_c_str(|ptr| gl::BindFragDataLocation(program, 0, ptr));

        let get_attrib_location = |s: &str| -> GLint { s.with_c_str(|ptr| gl::GetAttribLocation(program, ptr)) };
        let pos_attr = get_attrib_location("position");
        let color_attr = get_attrib_location("color");

        // Specify the layout of the vertex data
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::EnableVertexAttribArray(color_attr as GLuint);
        let stride = 6 * sizeof_float as GLsizei;
        gl::VertexAttribPointer(pos_attr as GLuint, 2, gl::FLOAT,
                                gl::FALSE, stride, ptr::null());
        gl::VertexAttribPointer(color_attr as GLuint, 4, gl::FLOAT,
                                gl::FALSE, stride, ptr::null().offset(2 * sizeof_float as int));
    }

    let get_uniform = |s: &str| -> GLint { s.with_c_str(|ptr| unsafe { gl::GetUniformLocation(program, ptr) }) };
    let modelview = get_uniform("modelview");

    let mut quat = Quaternion::identity();
    let mut velx: GLfloat = 0.0;
    let mut vely: GLfloat = 0.0;
    let mut velz: GLfloat = 0.0;

    while !window.should_close() {
        // Poll and handle events
        glfw.poll_events();
        handle_events(&window, &events);

        // Clear the screen to a nice black
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Rotate
        {
            velx += 0.001;
            vely -= 0.003;
            velz -= 0.001;
            quat = quat.mul_q(&Rotation3::from_angle_x(rad(velx)));
            quat = quat.mul_q(&Rotation3::from_angle_y(rad(vely)));
            quat = quat.mul_q(&Rotation3::from_angle_z(rad(velz)));
            uniform_quaternion(modelview, &quat);
        }

        // Draw a triangle from the 3 vertices
        gl::DrawArrays(gl::TRIANGLES, 0, 3);

        // Swap buffers
        window.swap_buffers();
    }

    // Cleanup
    gl::DeleteProgram(program);
    gl::DeleteShader(fs);
    gl::DeleteShader(vs);
    unsafe {
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}

fn uniform_quaternion(location: GLint, q: &Quaternion<GLfloat>) {
    let mat = q.to_matrix4();
    unsafe {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.ptr());
    }
}

fn handle_events(window: &glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => {
                window.set_should_close(true)
            },
            _ => {},
        }
    }
}
