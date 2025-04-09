use glutin::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use gl::types::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use colored::Colorize;
use num_cpus;

enum State {
    WaitingToStart,
    Stressing,
}

fn main() {
    print_ascii_art();
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("GPU Stresser");
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);
    let prog = compile_shaders();
    let (vao, _vbo) = create_quad();
    let stop = Arc::new(AtomicBool::new(false));
    let mut state = State::WaitingToStart;
    let mut threads = Vec::new();
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match state {
                            State::WaitingToStart => {
                                state = State::Stressing;
                                stop.store(false, Ordering::Relaxed);
                                let core_count = num_cpus::get();
                                for _ in 0..core_count {
                                    let s = stop.clone();
                                    threads.push(thread::spawn(move || cpu_stress(s)));
                                }
                                let s = stop.clone();
                                threads.push(thread::spawn(move || ram_stress(s)));
                            }
                            State::Stressing => {
                                if key == VirtualKeyCode::Q {
                                    stop.store(true, Ordering::Relaxed);
                                    for t in threads.drain(..) {
                                        t.join().unwrap();
                                    }
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => windowed_context.window().request_redraw(),
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                if let State::Stressing = state {
                    unsafe {
                        gl::UseProgram(prog);
                        gl::Uniform2f(gl::GetUniformLocation(prog, "resolution\0".as_ptr() as *const _), windowed_context.window().inner_size().width as f32, windowed_context.window().inner_size().height as f32);
                        gl::BindVertexArray(vao);
                        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                    }
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}

fn compile_shaders() -> GLuint {
    let vs_src = r#"
    #version 330 core
    layout (location = 0) in vec2 position;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
    "#;
    let fs_src = r#"
    #version 330 core
    out vec4 color;
    uniform vec2 resolution;
    void main() {
        vec2 uv = gl_FragCoord.xy / resolution;
        vec2 c = (uv * 4.0 - 2.0);
        c.y *= resolution.y / resolution.x;
        vec2 z = vec2(0.0);
        int max_iter = 1000;
        int iter = 0;
        while (dot(z, z) < 4.0 && iter < max_iter) {
            z = vec2(z.x*z.x - z.y*z.y, 2.0*z.x*z.y) + c;
            iter++;
        }
        float t = float(iter) / float(max_iter);
        color = vec4(t, t, t, 1.0);
    }
    "#;
    let vs = compile_shader(vs_src, gl::VERTEX_SHADER);
    let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER);
    let prog = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(prog, vs);
        gl::AttachShader(prog, fs);
        gl::LinkProgram(prog);
        let mut success = 0;
        gl::GetProgramiv(prog, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0; len as usize];
            gl::GetProgramInfoLog(prog, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut _);
            panic!("Program linking failed: {}", String::from_utf8_lossy(&buf));
        }
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
    }
    prog
}

fn compile_shader(src: &str, kind: GLenum) -> GLuint {
    let shader = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(shader, 1, [src.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0; len as usize];
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut _);
            panic!("Shader compilation failed: {}", String::from_utf8_lossy(&buf));
        }
    }
    shader
}

fn create_quad() -> (GLuint, GLuint) {
    let vertices: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as GLsizeiptr, vertices.as_ptr() as *const _, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 8, std::ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }
    (vao, vbo)
}

fn cpu_stress(stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        let mut sum = 0.0;
        for i in 0..1000000 {
            sum += (i as f64).powi(2);
        }
    }
}

fn ram_stress(stop: Arc<AtomicBool>) {
    let mut vec: Vec<u8> = vec![0; 512 * 1024 * 1024];
    while !stop.load(Ordering::Relaxed) {
        for i in 0..vec.len() {
            vec[i] = (i % 256) as u8;
        }
    }
}

fn print_ascii_art() {
    let art = r#"
    ██████╗ ██████╗ ██╗   ██╗    ███████╗████████╗██████╗ ███████╗███████╗███████╗███████╗██████╗ 
    ██╔════╝██╔═══██╗██║   ██║    ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██╔════╝██╔════╝██╔══██╗██╔══██╗
    ██║     ██║   ██║██║   ██║    ███████╗   ██║   ██████╔╝█████╗  ███████╗███████╗███████╗██████╔╝
    ██║     ██║   ██║██║   ██║    ╚════██║   ██║   ██╔══██╗██╔══╝  ╚════██║╚════██║██╔═══╝ ██╔══██╗
    ╚██████╗╚██████╔╝╚██████╔╝    ███████║   ██║   ██║  ██║███████╗███████║███████║██║     ██║  ██║
     ╚═════╝ ╚═════╝  ╚═════╝     ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚═╝     ╚═╝  ╚═╝
    "#;
    println!("{}", art.red().bold());
    println!("Warning: This will max out GPU, CPU, and RAM. Use with caution.");
    println!("Press any key in the window to start. Press 'q' to stop.");
}
