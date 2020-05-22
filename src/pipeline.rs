mod cache;

use crate::Region;
use cache::Cache;

use glow::HasContext;
use glyph_brush::rusttype::{point, Rect};

pub struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    instances: <glow::Context as HasContext>::Buffer,
    transform: <glow::Context as HasContext>::UniformLocation,
    sampler: <glow::Context as HasContext>::UniformLocation,
    cache: Cache,
    current_instances: usize,
    supported_instances: usize,
    current_transform: [f32; 16],
}

impl Pipeline {
    pub fn new(
        gl: &glow::Context,
        cache_width: u32,
        cache_height: u32,
    ) -> Pipeline {
        let cache = unsafe { Cache::new(gl, cache_width, cache_height) };

        let program = unsafe {
            create_program(
                gl,
                &[
                    (glow::VERTEX_SHADER, include_str!("./shader/vertex.vert")),
                    (
                        glow::FRAGMENT_SHADER,
                        include_str!("./shader/fragment.frag"),
                    ),
                ],
            )
        };

        let (vertex_array, instances) =
            unsafe { create_instance_buffer(gl, Instance::INITIAL_AMOUNT) };

        let transform = unsafe {
            gl.get_uniform_location(program, "transform")
                .expect("Get transform location")
        };

        let sampler = unsafe {
            gl.get_uniform_location(program, "font_sampler")
                .expect("Get sampler location")
        };

        unsafe {
            gl.use_program(Some(program));

            gl.uniform_matrix_4_f32_slice(
                Some(transform),
                false,
                &IDENTITY_MATRIX,
            );

            gl.use_program(None);
        }

        Pipeline {
            program,
            cache,
            vertex_array,
            instances,
            transform,
            sampler,
            current_instances: 0,
            supported_instances: Instance::INITIAL_AMOUNT,
            current_transform: IDENTITY_MATRIX,
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        transform: [f32; 16],
        region: Option<Region>,
    ) {
        unsafe {
            gl.use_program(Some(self.program));
        }

        if self.current_transform != transform {
            unsafe {
                gl.uniform_matrix_4_f32_slice(
                    Some(self.transform),
                    false,
                    &transform,
                );
            }

            self.current_transform = transform;
        }

        unsafe {
            if let Some(region) = region {
                gl.enable(glow::SCISSOR_TEST);
                gl.scissor(
                    region.x as i32,
                    region.y as i32,
                    region.width as i32,
                    region.height as i32,
                );
            }

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.cache.texture));
            gl.uniform_1_i32(Some(self.sampler), 0);

            gl.bind_vertex_array(Some(self.vertex_array));

            gl.draw_arrays_instanced(
                glow::TRIANGLE_STRIP,
                0,
                4,
                self.current_instances as i32,
            );

            gl.bind_vertex_array(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.disable(glow::SCISSOR_TEST);
            gl.use_program(None);
        }
    }

    pub fn update_cache(
        &mut self,
        gl: &glow::Context,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        unsafe {
            self.cache.update(gl, offset, size, data);
        }
    }

    pub fn increase_cache_size(
        &mut self,
        gl: &glow::Context,
        width: u32,
        height: u32,
    ) {
        unsafe {
            self.cache.destroy(gl);

            self.cache = Cache::new(gl, width, height);
        }
    }

    pub fn upload(&mut self, gl: &glow::Context, instances: &[Instance]) {
        if instances.is_empty() {
            self.current_instances = 0;
            return;
        }

        if instances.len() > self.supported_instances {
            unsafe {
                gl.delete_buffer(self.instances);
                gl.delete_vertex_array(self.vertex_array);
            }

            let (new_vertex_array, new_instances) =
                unsafe { create_instance_buffer(gl, instances.len()) };

            self.vertex_array = new_vertex_array;
            self.instances = new_instances;
            self.supported_instances = instances.len();
        }

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instances));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(instances),
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        self.current_instances = instances.len();
    }
}

// Helpers
#[cfg_attr(rustfmt, rustfmt_skip)]
const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Instance {
    left_top: [f32; 3],
    right_bottom: [f32; 2],
    tex_left_top: [f32; 2],
    tex_right_bottom: [f32; 2],
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Instance {}
unsafe impl bytemuck::Pod for Instance {}

impl Instance {
    const INITIAL_AMOUNT: usize = 50_000;
}

impl From<glyph_brush::GlyphVertex> for Instance {
    #[inline]
    fn from(vertex: glyph_brush::GlyphVertex) -> Instance {
        let glyph_brush::GlyphVertex {
            mut tex_coords,
            pixel_coords,
            bounds,
            color,
            z,
        } = vertex;

        let gl_bounds = bounds;

        let mut gl_rect = Rect {
            min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
            max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
        };

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if gl_rect.max.x > gl_bounds.max.x {
            let old_width = gl_rect.width();
            gl_rect.max.x = gl_bounds.max.x;
            tex_coords.max.x = tex_coords.min.x
                + tex_coords.width() * gl_rect.width() / old_width;
        }

        if gl_rect.min.x < gl_bounds.min.x {
            let old_width = gl_rect.width();
            gl_rect.min.x = gl_bounds.min.x;
            tex_coords.min.x = tex_coords.max.x
                - tex_coords.width() * gl_rect.width() / old_width;
        }

        if gl_rect.max.y > gl_bounds.max.y {
            let old_height = gl_rect.height();
            gl_rect.max.y = gl_bounds.max.y;
            tex_coords.max.y = tex_coords.min.y
                + tex_coords.height() * gl_rect.height() / old_height;
        }

        if gl_rect.min.y < gl_bounds.min.y {
            let old_height = gl_rect.height();
            gl_rect.min.y = gl_bounds.min.y;
            tex_coords.min.y = tex_coords.max.y
                - tex_coords.height() * gl_rect.height() / old_height;
        }

        Instance {
            left_top: [gl_rect.min.x, gl_rect.max.y, z],
            right_bottom: [gl_rect.max.x, gl_rect.min.y],
            tex_left_top: [tex_coords.min.x, tex_coords.max.y],
            tex_right_bottom: [tex_coords.max.x, tex_coords.min.y],
            color,
        }
    }
}

unsafe fn create_program(
    gl: &glow::Context,
    shader_sources: &[(u32, &str)],
) -> <glow::Context as HasContext>::Program {
    let program = gl.create_program().expect("Cannot create program");

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");

        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            panic!(gl.get_shader_info_log(shader));
        }

        gl.attach_shader(program, shader);

        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!(gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

unsafe fn create_instance_buffer(
    gl: &glow::Context,
    size: usize,
) -> (
    <glow::Context as HasContext>::VertexArray,
    <glow::Context as HasContext>::Buffer,
) {
    let vertex_array = gl.create_vertex_array().expect("Create vertex array");
    let buffer = gl.create_buffer().expect("Create instance buffer");

    gl.bind_vertex_array(Some(vertex_array));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
    gl.buffer_data_size(
        glow::ARRAY_BUFFER,
        (size * std::mem::size_of::<Instance>()) as i32,
        glow::DYNAMIC_DRAW,
    );

    let stride = std::mem::size_of::<Instance>() as i32;

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
    gl.vertex_attrib_divisor(0, 1);

    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 4 * 3);
    gl.vertex_attrib_divisor(1, 1);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, 4 * (3 + 2));
    gl.vertex_attrib_divisor(2, 1);

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_f32(
        3,
        2,
        glow::FLOAT,
        false,
        stride,
        4 * (3 + 2 + 2),
    );
    gl.vertex_attrib_divisor(3, 1);

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_f32(
        4,
        4,
        glow::FLOAT,
        false,
        stride,
        4 * (3 + 2 + 2 + 2),
    );
    gl.vertex_attrib_divisor(4, 1);

    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);

    (vertex_array, buffer)
}
