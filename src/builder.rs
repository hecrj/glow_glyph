use core::hash::BuildHasher;

use glyph_brush::delegate_glyph_brush_builder_fns;
use glyph_brush::{rusttype, DefaultSectionHasher};
use rusttype::{Error, Font, SharedBytes};

use super::GlyphBrush;

/// Builder for a [`GlyphBrush`](struct.GlyphBrush.html).
pub struct GlyphBrushBuilder<'a, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrushBuilder<'a, H>,
}

impl<'a, H> From<glyph_brush::GlyphBrushBuilder<'a, H>>
    for GlyphBrushBuilder<'a, H>
{
    fn from(inner: glyph_brush::GlyphBrushBuilder<'a, H>) -> Self {
        GlyphBrushBuilder { inner }
    }
}
impl<'a> GlyphBrushBuilder<'a> {
    /// Specifies the default font data used to render glyphs.
    /// Referenced with `FontId(0)`, which is default.
    #[inline]
    pub fn using_font_bytes<B: Into<SharedBytes<'a>>>(
        font_0_data: B,
    ) -> Result<Self, Error> {
        let font = Font::from_bytes(font_0_data)?;

        Ok(Self::using_font(font))
    }

    #[inline]
    pub fn using_fonts_bytes<B, V>(font_data: V) -> Result<Self, Error>
    where
        B: Into<SharedBytes<'a>>,
        V: Into<Vec<B>>,
    {
        let fonts = font_data
            .into()
            .into_iter()
            .map(Font::from_bytes)
            .collect::<Result<Vec<Font>, Error>>()?;

        Ok(Self::using_fonts(fonts))
    }

    /// Specifies the default font used to render glyphs.
    /// Referenced with `FontId(0)`, which is default.
    #[inline]
    pub fn using_font(font_0: Font<'a>) -> Self {
        Self::using_fonts(vec![font_0])
    }

    pub fn using_fonts<V: Into<Vec<Font<'a>>>>(fonts: V) -> Self {
        GlyphBrushBuilder {
            inner: glyph_brush::GlyphBrushBuilder::using_fonts(fonts),
        }
    }
}

impl<'a, H: BuildHasher> GlyphBrushBuilder<'a, H> {
    delegate_glyph_brush_builder_fns!(inner);

    /// Sets the section hasher. `GlyphBrush` cannot handle absolute section
    /// hash collisions so use a good hash algorithm.
    ///
    /// This hasher is used to distinguish sections, rather than for hashmap
    /// internal use.
    ///
    /// Defaults to [seahash](https://docs.rs/seahash).
    pub fn section_hasher<T: BuildHasher>(
        self,
        section_hasher: T,
    ) -> GlyphBrushBuilder<'a, T> {
        GlyphBrushBuilder {
            inner: self.inner.section_hasher(section_hasher),
        }
    }

    /// Builds a `GlyphBrush` using the given `wgpu::Device` that can render
    /// text for texture views with the given `render_format`.
    pub fn build(self, gl: &glow::Context) -> GlyphBrush<'a, H> {
        GlyphBrush::<H>::new(gl, self.inner)
    }
}
