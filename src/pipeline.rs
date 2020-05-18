mod cache;

use crate::Region;
use cache::Cache;

use glow::HasContext;
use glyph_brush::rusttype::{point, Rect};

pub struct Pipeline {
    sampler: <glow::Context as HasContext>::Sampler,
    program: <glow::Context as HasContext>::Program,
    instances: <glow::Context as HasContext>::Buffer,
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
        let sampler =
            unsafe { gl.create_sampler().expect("Create glyph sampler") };

        let cache = Cache::new(gl, cache_width, cache_height);

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

        let instances =
            unsafe { gl.create_buffer().expect("Create instance buffer") };

        Pipeline {
            sampler,
            program,
            cache,
            instances,
            current_instances: 0,
            supported_instances: Instance::INITIAL_AMOUNT,
            current_transform: [0.0; 16],
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        transform: [f32; 16],
        region: Option<Region>,
    ) {
    }

    pub fn update_cache(
        &mut self,
        gl: &glow::Context,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        self.cache.update(gl, offset, size, data);
    }

    pub fn increase_cache_size(
        &mut self,
        gl: &glow::Context,
        width: u32,
        height: u32,
    ) {
        self.cache = Cache::new(gl, width, height);
    }

    pub fn upload(&mut self, gl: &glow::Context, instances: &[Instance]) {
        if instances.is_empty() {
            self.current_instances = 0;
            return;
        }

        if instances.len() > self.supported_instances {
            // TODO

            self.supported_instances = instances.len();
        }

        // TODO

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
