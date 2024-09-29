use glow::{Context, HasContext};

fn load_shader(gl: &glow::Context, shader_type: u32, source: &str) -> glow::Shader {
    unsafe {
        let shader = gl.create_shader(shader_type).expect("Cannot create shader");

        let source = format!(
            "#version 300 es\n{}\n{}",
            if shader_type == glow::VERTEX_SHADER {
                "#define VS"
            } else {
                "#define FS"
            },
            source
        );

        gl.shader_source(shader, &source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        shader
    }
}

fn load_program(gl: &glow::Context, src: &str) -> glow::Program {
    unsafe {
        let program = gl.create_program().expect("Cannot create program");
        gl.attach_shader(program, load_shader(gl, glow::VERTEX_SHADER, src));
        gl.attach_shader(program, load_shader(gl, glow::FRAGMENT_SHADER, src));
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }
        program
    }
}

fn slice_to_u8_slice<'a, T>(data: &[T]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) }
}


pub fn init(gl: &Context) {
    let program = load_program(gl, &std::fs::read_to_string("shaders/crt.glsl").unwrap());
    unsafe {
        gl.use_program(Some(program));

        gl.enable(glow::TEXTURE_2D);
        gl.disable(glow::CULL_FACE);

        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            256,
            240,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vertices: [f32; 16] = [
            -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 0.0, 1.0,
        ];

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            slice_to_u8_slice(&vertices),
            glow::STATIC_DRAW,
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let ibo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            slice_to_u8_slice(&indices),
            glow::STATIC_DRAW,
        );

        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 4 * 4, 0);

        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 4 * 4, 2 * 4);
    }
}

pub fn render(gl: &Context, framebuffer: &[u32]) {
    let u8_pixels = slice_to_u8_slice(framebuffer);
    unsafe {
        gl.tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            256,
            240,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(u8_pixels),
        );
        gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_SHORT, 0);
    }
}

pub fn resize(gl: &Context, width: i32, height: i32) {
    let size = if width > height { height } else { width };
    unsafe {
        gl.viewport(width / 2 - size / 2, height / 2 - size / 2, size, size);
    }
}

pub struct GLBitmap {
    pub width: u16,
    pub height: u16,
    pub texture: glow::Texture,
}

impl GLBitmap {
    pub fn new(gl: &Context, width: u16, height: u16, data: &[u8]) -> Self {
        unsafe {
            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            Self {
                width,
                height,
                texture,
            }
        }
    }
}
