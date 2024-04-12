use std::ffi::{c_char, c_int, c_uint, c_void};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SubsetFont {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SubsetTextRun {
    pub offset: usize,
    pub length: usize,
    pub bidi_level: u8,
    pub script: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SubsetGlyphDrawer {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SubsetShapeParams {
    pub unichars: *const u32,
    pub unichar_count: usize,
    pub offset: usize,
    pub length: usize,
    pub bidi_level: u8,
    pub script: u32,
    pub language: *const c_char,
}

pub type SubsetFontProvider =
    Option<unsafe extern "C" fn(index: usize, context: *mut c_void) -> *mut SubsetFont>;

pub type SubsetTextRunCallback =
    Option<unsafe extern "C" fn(text_run: SubsetTextRun, context: *mut c_void)>;

pub type SubsetShapeCallback =
    Option<unsafe extern "C" fn(glyph: crate::Glyph, context: *mut c_void)>;

pub type SubsetPathCommandCallback =
    Option<unsafe extern "C" fn(verb: c_uint, points: *const f32, context: *mut c_void)>;

extern "C" {
    pub fn subset_font_draw_glyph(
        font: *mut SubsetFont,
        glyph_id: u32,
        drawer: *mut SubsetGlyphDrawer,
        callback: SubsetPathCommandCallback,
        context: *mut ::std::os::raw::c_void,
    );
}

extern "C" {
    pub fn subset_font_create_from_data(
        data: *const c_char,
        length: c_uint,
        index: c_uint,
    ) -> *mut SubsetFont;

    pub fn subset_font_create_from_file(filename: *const c_char, index: c_uint) -> *mut SubsetFont;

    pub fn subset_font_reference(font: *mut SubsetFont) -> *mut SubsetFont;

    pub fn subset_font_synthesize(
        font: *mut SubsetFont,
        embolden_strength: *const f32,
        slant: *const f32,
    ) -> *mut SubsetFont;

    pub fn subset_font_destroy(font: *mut SubsetFont);

    pub fn subset_font_has_glyph(font: *mut SubsetFont, unichar: u32) -> bool;

    pub fn subset_font_is_italic(font: *mut SubsetFont) -> bool;

    pub fn subset_font_weight(font: *mut SubsetFont) -> f32;

    pub fn subset_font_width(font: *mut SubsetFont) -> f32;

    pub fn subset_font_upem(font: *mut SubsetFont) -> c_uint;

    pub fn subset_font_extents(
        font: *mut SubsetFont,
        horizontal: bool,
        ascender: *mut i32,
        descender: *mut i32,
        line_gap: *mut i32,
    ) -> bool;

    pub fn subset_text_runs(
        unichars: *const u32,
        unichar_count: usize,
        paragraph_base_level: *mut u8,
        callback: SubsetTextRunCallback,
        context: *mut c_void,
    ) -> c_int;

    pub fn subset_find_best_font_match(
        unichar: u32,
        font_style: crate::FontStyle,
        font_count: usize,
        font_provider: SubsetFontProvider,
        font_provider_context: *mut c_void,
        best_index: *mut usize,
    ) -> bool;

    pub fn subset_glyph_drawer_create() -> *mut SubsetGlyphDrawer;

    pub fn subset_glyph_drawer_destroy(drawer: *mut SubsetGlyphDrawer);

    pub fn subset_shape(
        font: *mut SubsetFont,
        params: *mut SubsetShapeParams,
        callback: SubsetShapeCallback,
        context: *mut c_void,
    ) -> bool;
}
