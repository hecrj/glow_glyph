use glow::HasContext;

pub struct Cache {
    pub(crate) texture: <glow::Context as HasContext>::Texture,
}

impl Cache {
    pub unsafe fn new(gl: &glow::Context, width: u32, height: u32) -> Cache {
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

        let texture = {
            let handle =
                gl.create_texture().expect("Create glyph cache texture");

            gl.bind_texture(glow::TEXTURE_2D, Some(handle));

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

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            handle
        };

        Cache { texture }
    }

    pub unsafe fn update(
        &self,
        gl: &glow::Context,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        let [offset_x, offset_y] = offset;
        let [width, height] = size;

        gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

        gl.tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            i32::from(offset_x),
            i32::from(offset_y),
            i32::from(width),
            i32::from(height),
            glow::RED,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(data),
        );

        gl.bind_texture(glow::TEXTURE_2D, None);
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_texture(self.texture);
    }
}
