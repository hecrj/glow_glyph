use crate::ab_glyph::{point, Rect};
use crate::pipeline::cache::Cache;
use crate::Region;

use glow::HasContext;

pub struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertices: <glow::Context as HasContext>::Buffer,
    indices: <glow::Context as HasContext>::Buffer,
    transform: <glow::Context as HasContext>::UniformLocation,
    cache: Cache,
    current_vertices: usize,
    supported_vertices: usize,
    current_transform: [f32; 16],
    max_texture_size: u32,
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
                include_str!("../shader/compatibility/vertex.vert"),
                include_str!("../shader/compatibility/fragment.frag"),
            )
        };

        let vertex_array =
            unsafe { gl.create_vertex_array().expect("Create vertex array") };
        let (vertices, indices) =
            unsafe { create_buffers(gl, vertex_array, Vertex::INITIAL_AMOUNT) };

        let transform = unsafe {
            gl.get_uniform_location(program, "transform")
                .expect("Get transform location")
        };

        let sampler = unsafe {
            gl.get_uniform_location(program, "font_sampler")
                .expect("Get sampler location")
        };
        let max_texture_size = unsafe {
            match gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) {
                i32::MIN..=0 => 2048,
                size => size as u32,
            }
        };

        unsafe {
            gl.use_program(Some(program));

            gl.uniform_1_i32(Some(&sampler), 0);

            gl.uniform_matrix_4_f32_slice(
                Some(&transform),
                false,
                &IDENTITY_MATRIX,
            );

            gl.use_program(None);
        }

        Pipeline {
            program,
            cache,
            vertex_array,
            vertices,
            indices,
            transform,
            current_vertices: 0,
            supported_vertices: Vertex::INITIAL_AMOUNT,
            current_transform: IDENTITY_MATRIX,
            max_texture_size,
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
                    Some(&self.transform),
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

            gl.bind_vertex_array(Some(self.vertex_array));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertices));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.indices));

            gl.draw_elements(
                glow::TRIANGLES,
                (self.current_vertices as i32 * 3) / 2,
                glow::UNSIGNED_INT,
                0,
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

    pub fn upload(&mut self, gl: &glow::Context, vertices: &[[Vertex; 4]]) {
        // NOTE: Since we use `bytemuck::cast_slice` to convert our
        // vector of vertices to a byte slice, we don't need to flatten
        // the upload data (they are going to be bytes in the end anyway).
        //
        // But because of this, `vertices.len()` doesn't correspond to
        // the number of vertices anymore, so we use this variable for that.
        let vertex_count = vertices.len() * 4;

        if vertices.is_empty() {
            self.current_vertices = 0;
            return;
        }

        if vertex_count > self.supported_vertices {
            unsafe {
                gl.delete_buffer(self.vertices);
                gl.delete_vertex_array(self.vertex_array);
            }

            let (vertex_buffer, index_buffer) =
                unsafe { create_buffers(gl, self.vertex_array, vertex_count) };

            self.vertices = vertex_buffer;
            self.indices = index_buffer;
            self.supported_vertices = vertex_count;
        }

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertices));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(vertices),
            );

            let indices = (0..vertex_count as i32).fold(
                Vec::with_capacity(vertex_count),
                |mut indices, i| {
                    indices.extend_from_slice(&[
                        0 + i * 4,
                        1 + i * 4,
                        2 + i * 4,
                        2 + i * 4,
                        1 + i * 4,
                        3 + i * 4,
                    ]);
                    indices
                },
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.indices));
            gl.buffer_sub_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(&indices),
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        self.current_vertices = vertex_count;
    }

    pub fn get_max_texture_size(&self) -> u32 {
        self.max_texture_size
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
pub struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    extra: f32,
    color: [f32; 4],
}

impl Vertex {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    const INITIAL_AMOUNT: usize = 50_000 * 4; // 200_000 vertices (or, 50_000 glyphs)

    pub fn from_vertex(
        glyph_brush::GlyphVertex {
            mut tex_coords,
            pixel_coords,
            bounds,
            extra,
        }: &glyph_brush::GlyphVertex,
    ) -> [Vertex; 4] {
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

        // NOTE: This makes so that one `glyph` corresponds
        // to four vertices, which then makes one quad.
        // This is used for maximum compatibility, where
        // some hardware don't support instancing.
        // e.g. OpenGL 2.1, OpenGL ES 2.0, etc.
        [
            Vertex {
                pos: [gl_rect.min.x, gl_rect.max.y],
                uv: [tex_coords.min.x, tex_coords.max.y],
                extra: extra.z,
                color: extra.color,
            },
            Vertex {
                pos: [gl_rect.max.x, gl_rect.max.y],
                uv: [tex_coords.max.x, tex_coords.max.y],
                extra: extra.z,
                color: extra.color,
            },
            Vertex {
                pos: [gl_rect.min.x, gl_rect.min.y],
                uv: [tex_coords.min.x, tex_coords.min.y],
                extra: extra.z,
                color: extra.color,
            },
            Vertex {
                pos: [gl_rect.max.x, gl_rect.min.y],
                uv: [tex_coords.max.x, tex_coords.min.y],
                extra: extra.z,
                color: extra.color,
            },
        ]
    }
}

unsafe fn create_program(
    gl: &glow::Context,
    vertex_source: &str,
    fragment_source: &str,
) -> <glow::Context as HasContext>::Program {
    let version = gl.version();

    // This may look crazy, but in essence it defines:
    // - Version directive
    // - Fragment shader output (if needed)
    // - Changes legacy `attribute` and `varying` to `in` and `out`
    //
    // By doing so, we make sure to define the shader only once,
    // and make it work across different versions.
    let (vertex_version, fragment_version) =
        match (version.major, version.minor, version.is_embedded) {
            // OpenGL 3.0+
            (3, 0 | 1 | 2, false) => (
                format!("#version 1{}0\n#extension GL_ARB_explicit_attrib_location : enable", version.minor + 3),
                format!(
                    "#version 1{}0\n#define HIGHER_THAN_300 1",
                    version.minor + 3
                ),
            ),
            // OpenGL 3.3+
            (3 | 4, _, false) => (
                format!("#version {}{}0", version.major, version.minor),
                format!(
                    "#version {}{}0\n#define HIGHER_THAN_300 1",
                    version.major, version.minor
                ),
            ),
            // OpenGL ES 3.0+
            (3, _, true) => (
                format!("#version 3{}0 es", version.minor),
                format!(
                    "#version 3{}0 es\n#define HIGHER_THAN_300 1",
                    version.minor
                ),
            ),
            // OpenGL ES 2.0+
            (2, _, true) => (
                String::from(
                    "#version 100\n#define in attribute\n#define out varying",
                ),
                String::from("#version 100\n#define in varying"),
            ),
            // OpenGL 2.1
            (2, _, false) => (
                String::from(
                    "#version 120\n#define in attribute\n#define out varying\n#extension GL_ARB_explicit_attrib_location : enable",
                ),
                String::from("#version 120\n#define in varying"),
            ),
            // OpenGL 1.1+
            _ => panic!("Incompatible context version: {:?}", version),
        };
    log::info!(
        "Shader directive: {}",
        vertex_version.lines().next().unwrap()
    );

    let shader_sources = [
        (
            glow::VERTEX_SHADER,
            &format!("{}\n{}", vertex_version, vertex_source),
        ),
        (
            glow::FRAGMENT_SHADER,
            &format!("{}\n{}", fragment_version, fragment_source),
        ),
    ];

    let program = gl.create_program().expect("Cannot create program");

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");

        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }

        gl.attach_shader(program, shader);

        shaders.push(shader);
    }

    gl.bind_attrib_location(program, 0, "pos");
    gl.bind_attrib_location(program, 1, "uv");
    gl.bind_attrib_location(program, 2, "extra");
    gl.bind_attrib_location(program, 3, "color");

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

unsafe fn create_buffers(
    gl: &glow::Context,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    buffer_size: usize,
) -> (
    <glow::Context as HasContext>::Buffer,
    <glow::Context as HasContext>::Buffer,
) {
    gl.bind_vertex_array(Some(vertex_array));

    let vertex_buffer = gl.create_buffer().expect("Create vertex buffer");
    let index_buffer = gl.create_buffer().expect("Create index buffer");

    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
    gl.buffer_data_size(
        glow::ARRAY_BUFFER,
        (buffer_size * Vertex::SIZE) as i32,
        glow::DYNAMIC_DRAW,
    );

    // For every 4 vertices, we have 6 indices
    // The indices are bytes, which have size 4
    // Making the buffer size: `buffer_size * (6/4) * 4` bytes
    // Or simply: `buffer_size * 6` bytes
    let index_buffer_size = buffer_size as i32 * 6;
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
    gl.buffer_data_size(
        glow::ELEMENT_ARRAY_BUFFER,
        index_buffer_size,
        glow::DYNAMIC_DRAW,
    );

    // vec2 pos;
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(
        0,
        2,
        glow::FLOAT,
        false,
        Vertex::SIZE as i32,
        0,
    );

    // vec2 uv;
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(
        1,
        2,
        glow::FLOAT,
        false,
        Vertex::SIZE as i32,
        4 * 2,
    );

    // float extra;
    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_f32(
        2,
        1,
        glow::FLOAT,
        false,
        Vertex::SIZE as i32,
        4 * (2 + 2),
    );

    // vec4 color;
    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_f32(
        3,
        4,
        glow::FLOAT,
        false,
        Vertex::SIZE as i32,
        4 * (2 + 2 + 1),
    );

    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);
    gl.bind_vertex_array(None);

    (vertex_buffer, index_buffer)
}
