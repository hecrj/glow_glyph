# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2022-05-10
### Fixed
- `GL_ARB_explicit_attrib_location` not being explicitly enabled in `core` shaders. [#7]

[#7]: https://github.com/hecrj/glow_glyph/pull/7

## [0.5.0] - 2021-12-17
### Added
- Support for older hardware. [#5]

### Changed
- `glow` updated to `0.11.1`. [#5]

[#5]: https://github.com/hecrj/glow_glyph/pull/5

## [0.4.0] - 2020-11-10
### Changed
- `glow` updated to `0.6`. [#3]
- `bytemuck` updated to `1.4`. [#3]

[#3]: https://github.com/hecrj/glow_glyph/pull/3

## [0.3.0] - 2020-07-27
### Changed
- `glow` updated to `0.5`. [#2]

[#2]: https://github.com/hecrj/glow_glyph/pull/2


## [0.2.1] - 2020-06-27
### Fixed
- Removed object label for `Cache`. `glObjectLabel` is not widely supported, causing panics in some environments.

## [0.2.0] - 2020-05-27
### Changed
- `glyph_brush` updated to `0.7`. [#1]

[#1]: https://github.com/hecrj/glow_glyph/pull/1


## [0.1.0] - 2020-05-22
### Added
- First release! :tada:


[Unreleased]: https://github.com/hecrj/glow_glyph/compare/0.5.1...HEAD
[0.5.1]: https://github.com/hecrj/glow_glyph/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/hecrj/glow_glyph/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/hecrj/glow_glyph/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/hecrj/glow_glyph/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/hecrj/glow_glyph/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/hecrj/glow_glyph/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/hecrj/glow_glyph/releases/tag/0.1.0
