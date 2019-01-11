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

use image;


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
    let pixels = loop {
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
    let num_values = (2 as f32).powi(((w * h) as f32).log2().ceil() as i32) as usize;
    println!("{:#?}", w * h);
    println!("{}", num_values);

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
              UniformBuffer::empty_unsized(display, 4*4 * num_values).unwrap();

    {
        let mut mapping = buffer.map();
        for (i, val) in mapping.values.iter_mut().enumerate() {
            // println!("{:?}", val);
            // *val = rand::random::<f32>();
            let start = i * 4;
            if start < (w * h) * 4 {
                *val = [
                    pixels[start + 2] as f32 / 255.,
                    pixels[start + 1] as f32 / 255.,
                    pixels[start] as f32 / 255.,
                    pixels[start + 3] as f32 / 255.,
                ];
            }
            else {
                *val = [0.;4]
            }
        }
    }

    const THREAD_SIZE: u32 = 256;
    let work_group_size_y = (h as f32 / 2. / THREAD_SIZE as f32).ceil() as u32;
    println!("w: {}, h: {}, work_group_size_y: {}", w, h, work_group_size_y);

    let program = load_shader(display, &PathBuf::from("shaders/sort.glsl"));

    for block in 0..((w as f32).log(2.) as i32 + 1) {
        for iteration in (0..block).rev() {
            program.execute(uniform! {
                ToSort: &*buffer,
                current_block: block,
                current_iteration: iteration,
                width: w as u32,
                height: h as u32,
            }, w as u32, work_group_size_y, 1);
        }
    }


    let mut imgbuf = image::ImageBuffer::new(w as u32, h as u32);
    {
        let mapping = buffer.map();
        for x in 0..(w as u32) {
            for y in 0..(h as u32) {
                let target = (x+ y * w as u32 ) as usize;
                let values_u8 = [
                    (mapping.values[target][0] * 255.) as u8,
                    (mapping.values[target][1] * 255.) as u8,
                    (mapping.values[target][2] * 255.) as u8,
                    (mapping.values[target][3] * 255.) as u8,
                ];
                *imgbuf.get_pixel_mut(x, y) = image::Rgba(values_u8);
            }
        }
    }
    imgbuf.save("output.png");
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
