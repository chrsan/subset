extern crate link_cplusplus;

mod ffi;
mod font;
mod layout;

pub use self::font::*;
pub use self::layout::*;

#[derive(Debug, Clone, Copy)]
pub struct FontExtents {
    pub ascender: i32,
    pub descender: i32,
    pub line_gap: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontStyle {
    pub italic: bool,
    pub weight: f32,
    pub width: f32,
}

impl FontStyle {
    pub fn bold() -> Self {
        Self {
            weight: 700.0,
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: true,
            ..Default::default()
        }
    }

    pub fn bold_italic() -> Self {
        Self {
            italic: true,
            weight: 700.0,
            ..Default::default()
        }
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            italic: false,
            weight: 400.0,
            width: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontRun {
    pub offset: usize,
    pub len: usize,
    pub bidi_level: u8,
    pub script: u32,
    pub font_index: usize,
    pub font_style: FontStyle,
    pub synthetic_bold: bool,
    pub synthetic_slant: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Glyph {
    pub glyph_id: u32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub x_advance: i32,
    pub y_advance: i32,
}

#[derive(Debug, Clone)]
pub struct GlyphRun {
    pub font_run_index: usize,
    pub glyphs: Vec<Glyph>,
    pub paths: Vec<Path>,
}

#[derive(Debug, Clone, Copy)]
pub enum PathVerb {
    MoveTo,
    LineTo,
    QuadTo,
    CubicTo,
    Close,
}

impl PathVerb {
    pub fn num_points(&self) -> usize {
        match self {
            Self::MoveTo | Self::LineTo => 1,
            Self::QuadTo => 2,
            Self::CubicTo => 3,
            Self::Close => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathCommand {
    MoveTo((f32, f32)),
    LineTo((f32, f32)),
    QuadTo((f32, f32), (f32, f32)),
    CubicTo((f32, f32), (f32, f32), (f32, f32)),
    Close,
}

#[derive(Debug, Default, Clone)]
pub struct Path {
    pub verbs: Vec<PathVerb>,
    pub points: Vec<(f32, f32)>,
}

impl Path {
    pub fn iter(&self) -> PathIter<'_> {
        PathIter {
            path: self,
            verb_index: 0,
            point_index: 0,
        }
    }
}
#[derive(Debug)]
pub struct PathIter<'a> {
    path: &'a Path,
    verb_index: usize,
    point_index: usize,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathCommand;

    fn next(&mut self) -> Option<Self::Item> {
        let verb = self.path.verbs.get(self.verb_index)?;
        self.verb_index += 1;
        match verb {
            PathVerb::MoveTo => {
                let point = self.path.points.get(self.point_index)?;
                self.point_index += 1;
                Some(PathCommand::MoveTo(*point))
            }
            PathVerb::LineTo => {
                let point = self.path.points.get(self.point_index)?;
                self.point_index += 1;
                Some(PathCommand::LineTo(*point))
            }
            PathVerb::QuadTo => {
                let points = self
                    .path
                    .points
                    .get(self.point_index..self.point_index + 2)?;
                self.point_index += 2;
                Some(PathCommand::QuadTo(points[0], points[1]))
            }
            PathVerb::CubicTo => {
                let points = self
                    .path
                    .points
                    .get(self.point_index..self.point_index + 3)?;
                self.point_index += 3;
                Some(PathCommand::CubicTo(points[0], points[1], points[3]))
            }
            PathVerb::Close => Some(PathCommand::Close),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnicodeValue {
    Char(char),
    Codepoint(u32),
}

impl From<UnicodeValue> for u32 {
    fn from(value: UnicodeValue) -> Self {
        match value {
            UnicodeValue::Char(v) => v as u32,
            UnicodeValue::Codepoint(v) => v,
        }
    }
}

impl From<char> for UnicodeValue {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl From<u32> for UnicodeValue {
    fn from(value: u32) -> Self {
        Self::Codepoint(value)
    }
}

impl From<&u32> for UnicodeValue {
    fn from(value: &u32) -> Self {
        Self::Codepoint(*value)
    }
}
