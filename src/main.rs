use glium;
use rand;

use std::fs::File;
use std::io::{
    prelude::*,
    ErrorKind::WouldBlock
};

use std::path::{PathBuf, Path};

use glium::{
    // Macros
    implement_buffer_content,
    implement_uniform_block,
    uniform,
    // Children
    glutin::{
        self,
    },
    uniforms::UniformBuffer,
    program::{
        ProgramCreationError,
        ComputeShader
    },
    backend::Facade,
};

use scrap::{Capturer, Display};


fn load_shader(facade: &impl Facade, filename: &Path) -> ComputeShader {
    let mut file = File::open(filename).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read file");

    match ComputeShader::from_source(facade, &contents) {
        Ok(program) => program,
        Err(ProgramCreationError::CompilationError(message)) => {
            panic!("Shader compilation error:\n{}", message);
        }
        Err(ProgramCreationError::LinkingError(message)) => {
            panic!("Shader link error:\n{}", message);
        },
        other => {
            other.expect("Shader compilation failed");
            unreachable!()
        }
    }
}


fn sorted_screenshot(display: &impl Facade) {
    let screenshot_display = Display::primary().expect("Couldn't open display");
    let mut capturer = Capturer::new(screenshot_display).expect("Couldn't begin capture");
    let (w, h) = (capturer.width(), capturer.height());
    // Capture a screenshot
    let buffer = loop {
        match capturer.frame() {
            Ok(buffer) => break buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    continue;
                }
                else {
                    panic!("Error: {}", error)
                }
            }
        }
    };

    struct Data {
        values: [[f32; 4]],
    }

    implement_buffer_content!(Data);
    implement_uniform_block!(Data, values);

    // const NUM_VALUES: usize = 32;
    let num_values = (2 as f32).powi(((w * h) as f32).log2() as i32) as usize;

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
              UniformBuffer::empty_unsized(display, 4*4 * num_values).unwrap();

    {
        let mut mapping = buffer.map();
        for (_, val) in mapping.values.iter_mut().enumerate() {
            // println!("{:?}", val);
            // *val = rand::random::<f32>();
        }
    }

    const THREAD_SIZE: u32 = 256;

    let program = load_shader(display, &PathBuf::from("shaders/sort.glsl"));

    for block in 0..((num_values as f32).log(2.) as i32 + 1) {
        for iteration in (0..block).rev() {
            program.execute(uniform! {
                ToSort: &*buffer,
                current_block: block,
                current_iteration: iteration
            }, num_values as u32 / 2 / THREAD_SIZE, 1, 1);
        }
    }

    {
        let mapping = buffer.map();
        for val in mapping.values.iter().take(9) {
            println!("{:?}", val);
        }
        println!("...");
    }
}


fn prefix_sum(display: &impl Facade) {
    struct Data {
        values: [f32],
    }

    implement_buffer_content!(Data);
    implement_uniform_block!(Data, values);

    const NUM_VALUES: usize = 16;

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
              UniformBuffer::empty_unsized(display, 4 * NUM_VALUES).unwrap();

    {
        let mut mapping = buffer.map();
        for (i, val) in mapping.values.iter_mut().enumerate() {
            *val = i as f32;
            // *val = 0.;
        }
    }

    let program = load_shader(display, &PathBuf::from("shaders/prefix_sum.glsl"));

    for i in 0..((NUM_VALUES as f32).log(2.) as u32) {
        program.execute(uniform! {
            Input: &*buffer,
            iteration: i
        }, NUM_VALUES as u32 / 2, 1, 1);
    }

    {
        let mapping = buffer.map();
        for val in mapping.values.iter() {
            println!("{:?}", val);
        }
        println!("...");
    }
}

fn main() {
    let event_loop = glutin::EventsLoop::new();
    let context_builder = glutin::ContextBuilder::new();
    let window = glutin::WindowBuilder::new().with_visibility(false)
        .with_visibility(false);
    let display = glium::Display::new(window, context_builder, &event_loop)
        .unwrap();

    sorted_screenshot(&display);
    // prefix_sum(&display);
}
