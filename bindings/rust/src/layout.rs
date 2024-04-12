use std::borrow::Cow;
use std::collections::VecDeque;
use std::ffi::{c_uint, c_void};
use std::ops::Range;
use std::{iter, ptr, slice};

use crate::{
    ffi, find_best_font_match, Font, FontRun, FontStyle, Glyph, GlyphRun, Path, PathVerb, Syntesize,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ShapeParams {
    pub embolden_strength: Option<f32>,
    pub slant: Option<f32>,
    pub emit_path_commands: bool,
}

#[derive(Debug, Clone)]
pub struct Layout<'a> {
    fonts: &'a [Font],
    codepoints: Vec<u32>,
    runs: Vec<FontRun>,
    paragraph_base_level: u8,
}

impl<'a> Layout<'a> {
    pub fn fonts(&self) -> &[Font] {
        self.fonts
    }

    pub fn runs(&self) -> &[FontRun] {
        &self.runs
    }

    pub fn paragraph_base_level(&self) -> u8 {
        self.paragraph_base_level
    }

    pub fn shape(&self, params: ShapeParams) -> Vec<GlyphRun> {
        if self.codepoints.is_empty() {
            return Vec::new();
        }
        let glyph_drawer = if params.emit_path_commands {
            Some(GlyphDrawer::new())
        } else {
            None
        };
        let mut runs = Vec::new();
        for (font_run_index, font_run) in self.runs.iter().enumerate() {
            let font = &self.fonts[font_run.font_index];
            let synthesize = match (
                font_run.synthetic_bold,
                font_run.synthetic_slant,
                params.embolden_strength,
                params.slant,
            ) {
                (true, false, Some(embolden_strength), _) => {
                    Some(Syntesize::Embolden(embolden_strength))
                }
                (false, true, _, Some(slant)) => Some(Syntesize::Slant(slant)),
                (true, true, Some(embolden_strength), Some(slant)) => {
                    Some(Syntesize::EmboldenAndSlant {
                        embolden_strength,
                        slant,
                    })
                }
                (true, true, Some(embolden_strength), _) => {
                    Some(Syntesize::Embolden(embolden_strength))
                }
                (true, true, _, Some(slant)) => Some(Syntesize::Slant(slant)),
                _ => None,
            };
            let font = if let Some(synthesize) = synthesize {
                let font = font.synthesize(synthesize);
                Cow::Owned(font)
            } else {
                Cow::Borrowed(font)
            };
            let (glyphs, paths) = shape(&self.codepoints, &font, font_run, glyph_drawer.as_ref());
            runs.push(GlyphRun {
                font_run_index,
                glyphs,
                paths,
            });
        }
        runs
    }
}

#[derive(Debug, Clone)]
pub struct LayoutBuilder<'a> {
    fonts: &'a [Font],
    codepoints: Vec<u32>,
    styles: Vec<FontStyle>,
    style_indices: Vec<usize>,
}

impl<'a> LayoutBuilder<'a> {
    pub fn new(fonts: &'a [Font]) -> Self {
        assert!(!fonts.is_empty());
        Self {
            fonts,
            codepoints: Vec::new(),
            styles: Vec::new(),
            style_indices: Vec::new(),
        }
    }

    pub fn fonts(&self) -> &[Font] {
        self.fonts
    }

    pub fn clear(&mut self) {
        self.codepoints.clear();
        self.styles.clear();
        self.style_indices.clear();
    }

    pub fn push(&mut self, text: impl Iterator<Item = char>, style: FontStyle) {
        let start = self.codepoints.len();
        self.codepoints.extend(text.map(|c| c as u32));
        let end = self.codepoints.len();
        let style_index = self.styles.len();
        self.styles.push(style);
        self.style_indices
            .extend(iter::repeat(style_index).take(end - start));
    }

    pub fn has_missing_glyphs(&self) -> bool {
        for v in self.codepoints.iter() {
            if !self.fonts.iter().any(|font| font.has_glyph(v)) {
                return true;
            }
        }
        !self.codepoints.is_empty() && self.fonts.is_empty()
    }

    pub fn build(self) -> Layout<'a> {
        if self.codepoints.is_empty() {
            Layout {
                fonts: self.fonts,
                codepoints: self.codepoints,
                runs: Vec::new(),
                paragraph_base_level: 0,
            }
        } else {
            let mut paragraph_base_level = 0u8;
            let runs = compute_runs(&self, &mut paragraph_base_level);
            Layout {
                fonts: self.fonts,
                codepoints: self.codepoints,
                runs,
                paragraph_base_level,
            }
        }
    }
}

fn compute_runs(builder: &LayoutBuilder<'_>, paragraph_base_level: &mut u8) -> Vec<FontRun> {
    struct Context<'a> {
        builder: &'a LayoutBuilder<'a>,
        runs: Vec<FontRun>,
    }
    unsafe extern "C" fn text_run_callback(run: ffi::SubsetTextRun, context: *mut c_void) {
        let Context { builder, runs } = &mut *(context as *mut Context<'_>);
        let rtl = (run.bidi_level & 1) != 0;
        let mut deque = VecDeque::new();
        for (offset, len, style) in
            split_run(run.offset, run.length, FontStyle::default(), |index| {
                builder.styles[builder.style_indices[index]]
            })
        {
            for (offset, len, index) in split_run(offset, len, 0, |index| {
                let codepoint = builder.codepoints[index];
                find_best_font_match(builder.fonts, codepoint, style).unwrap_or(0)
            }) {
                let font = &builder.fonts[index];
                let FontStyle { italic, weight, .. } = font.style();
                let run = FontRun {
                    offset,
                    len,
                    bidi_level: run.bidi_level,
                    script: run.script,
                    font_index: index,
                    font_style: style,
                    synthetic_bold: style.weight > weight,
                    synthetic_slant: style.italic && !italic,
                };
                if rtl {
                    deque.push_front(run);
                } else {
                    deque.push_back(run);
                }
            }
        }
        runs.extend(deque);
    }
    let mut context = Context {
        builder,
        runs: Vec::new(),
    };
    unsafe {
        ffi::subset_text_runs(
            builder.codepoints.as_ptr(),
            builder.codepoints.len(),
            paragraph_base_level as *mut _,
            Some(text_run_callback),
            &mut context as *mut _ as *mut _,
        );
    }
    context.runs
}

struct SplitRun<T, F> {
    done: bool,
    range: Range<usize>,
    offset: usize,
    len: usize,
    last_value: T,
    f: F,
}

fn split_run<T, F>(offset: usize, len: usize, last_value: T, f: F) -> SplitRun<T, F> {
    SplitRun {
        done: false,
        range: offset..offset + len,
        offset,
        len: 0,
        last_value,
        f,
    }
}

impl<T, F> Iterator for SplitRun<T, F>
where
    T: Copy + PartialEq,
    F: FnMut(usize) -> T,
{
    type Item = (usize, usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        for index in self.range.by_ref() {
            let new_value = (self.f)(index);
            if self.len != 0 && new_value != self.last_value {
                let item = (self.offset, self.len, self.last_value);
                self.offset = index;
                self.len = 1;
                self.last_value = new_value;
                return Some(item);
            }
            self.len += 1;
            self.last_value = new_value;
        }
        self.done = true;
        Some((self.offset, self.len, self.last_value))
    }
}

struct GlyphDrawer(*mut ffi::SubsetGlyphDrawer);

impl GlyphDrawer {
    fn new() -> Self {
        let raw = unsafe { ffi::subset_glyph_drawer_create() };
        assert!(!raw.is_null());
        Self(raw)
    }
}

impl Drop for GlyphDrawer {
    fn drop(&mut self) {
        unsafe {
            ffi::subset_glyph_drawer_destroy(self.0);
        }
    }
}

fn shape(
    codepoints: &[u32],
    font: &Font,
    run: &FontRun,
    glyph_drawer: Option<&GlyphDrawer>,
) -> (Vec<Glyph>, Vec<Path>) {
    struct Context {
        font: *mut ffi::SubsetFont,
        glyph_drawer: *mut ffi::SubsetGlyphDrawer,
        glyphs: Vec<Glyph>,
        paths: Vec<Path>,
    }
    unsafe extern "C" fn path_command_callback(
        verb: c_uint,
        points: *const f32,
        context: *mut c_void,
    ) {
        let path: &mut Path = &mut *(context as *mut Path);
        let num_points = match verb {
            0 => {
                path.verbs.push(PathVerb::MoveTo);
                1
            }
            1 => {
                path.verbs.push(PathVerb::LineTo);
                1
            }
            2 => {
                path.verbs.push(PathVerb::QuadTo);
                2
            }
            3 => {
                path.verbs.push(PathVerb::CubicTo);
                3
            }
            4 => {
                path.verbs.push(PathVerb::Close);
                0
            }
            _ => {
                unreachable!();
            }
        };
        if num_points != 0 {
            path.points.extend(
                slice::from_raw_parts(points, num_points * 2)
                    .chunks_exact(2)
                    .map(|chunk| (chunk[0], chunk[1])),
            );
        }
    }
    unsafe extern "C" fn shape_callback(glyph: crate::Glyph, context: *mut c_void) {
        let Context {
            font,
            glyph_drawer,
            glyphs,
            paths,
        } = &mut *(context as *mut Context);
        glyphs.push(glyph);
        if !glyph_drawer.is_null() {
            let mut path = Path::default();
            ffi::subset_font_draw_glyph(
                *font,
                glyph.glyph_id,
                *glyph_drawer,
                Some(path_command_callback),
                &mut path as *mut _ as *mut _,
            );
            paths.push(path);
        }
    }
    let mut params = ffi::SubsetShapeParams {
        unichars: codepoints.as_ptr(),
        unichar_count: codepoints.len(),
        offset: run.offset,
        length: run.len,
        bidi_level: run.bidi_level,
        script: run.script,
        language: ptr::null(), // TODO: Fix me!
    };
    let glyph_drawer = if let Some(drawer) = glyph_drawer {
        drawer.0
    } else {
        ptr::null_mut()
    };
    let mut context = Context {
        font: font.0,
        glyph_drawer,
        glyphs: Vec::new(),
        paths: Vec::new(),
    };
    unsafe {
        ffi::subset_shape(
            font.0,
            &mut params as *mut _,
            Some(shape_callback),
            &mut context as *mut _ as *mut _,
        );
    }
    (context.glyphs, context.paths)
}
