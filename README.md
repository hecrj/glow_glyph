# glow_glyph

[![Integration status](https://github.com/hecrj/glow_glyph/workflows/Integration/badge.svg)](https://github.com/hecrj/glow_glyph/actions)
[![crates.io](https://img.shields.io/crates/v/glow_glyph.svg)](https://crates.io/crates/glow_glyph)
[![Documentation](https://docs.rs/glow_glyph/badge.svg)](https://docs.rs/glow_glyph)
[![License](https://img.shields.io/crates/l/glow_glyph.svg)](https://github.com/hecrj/glow_glyph/blob/master/LICENSE)

A fast text renderer for [glow](https://github.com/grovesNL/glow), powered by
[glyph_brush](https://github.com/alexheretic/glyph-brush/tree/master/glyph-brush)

```rust
use glow_glyph::{Section, GlyphBrushBuilder};

let font: &[u8] = include_bytes!("SomeFont.ttf");
let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(font)
    .expect("Load font")
    .build(&glow_context);

let section = Section {
    text: "Hello glow_glyph",
    ..Section::default() // color, position, etc
};

glyph_brush.queue(section);
glyph_brush.queue(some_other_section);

glyph_brush.draw_queued(
    &glow_context,
    window_width,
    window_height,
);
```

## Examples

Have a look at
  * `cargo run --example hello`
  * `cargo run --example clipping`
