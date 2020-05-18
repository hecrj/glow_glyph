use glow::HasContext;

pub struct Cache {
    texture: <glow::Context as HasContext>::Texture,
}

impl Cache {
    pub fn new(gl: &glow::Context, width: u32, height: u32) -> Cache {
        let texture = unsafe {
            let handle = gl.create_texture().expect("Create glyph cache texture");

            gl.tex_storage_2d(handle, 1, glow::R8, width as i32, height as i32);

            handle
        };

        Cache { texture }
    }

    pub fn update(&self, gl: &glow::Context, offset: [u16; 2], size: [u16; 2], data: &[u8]) {
        // TODO
    }
}
