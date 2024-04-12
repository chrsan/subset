use std::ffi::{c_uint, c_void, CString};
use std::fmt::{self, Debug};
use std::path::Path;
use std::ptr;

use crate::{ffi, FontExtents, FontStyle, UnicodeValue};

#[derive(Debug, Clone, Copy)]
pub enum Syntesize {
    Embolden(f32),
    Slant(f32),
    EmboldenAndSlant { embolden_strength: f32, slant: f32 },
}

pub struct Font(pub(crate) *mut ffi::SubsetFont);

impl Font {
    pub fn from_data(data: impl AsRef<[u8]>, index: u32) -> Option<Self> {
        let data = data.as_ref();
        assert!(data.len() <= c_uint::MAX as _);
        let raw = unsafe {
            ffi::subset_font_create_from_data(data.as_ptr() as *const _, data.len() as _, index)
        };
        if raw.is_null() {
            None
        } else {
            Some(Font(raw))
        }
    }

    pub fn from_file(path: &Path, index: u32) -> Option<Self> {
        let cs = CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
        let raw = unsafe { ffi::subset_font_create_from_file(cs.as_ptr(), index) };
        if raw.is_null() {
            None
        } else {
            Some(Font(raw))
        }
    }

    pub fn synthesize(&self, synthesize: Syntesize) -> Self {
        let (embolden_strength, slant) = match synthesize {
            Syntesize::Embolden(ref embolden_strength) => {
                (embolden_strength as *const _, ptr::null())
            }
            Syntesize::Slant(ref slant) => (ptr::null(), slant as *const _),
            Syntesize::EmboldenAndSlant {
                ref embolden_strength,
                ref slant,
            } => (embolden_strength as *const _, slant as *const _),
        };
        let raw = unsafe { ffi::subset_font_synthesize(self.0, embolden_strength, slant) };
        assert!(!raw.is_null());
        Self(raw)
    }

    pub fn has_glyph(&self, value: impl Into<UnicodeValue>) -> bool {
        let value = match value.into() {
            UnicodeValue::Char(v) => v as u32,
            UnicodeValue::Codepoint(v) => v,
        };
        unsafe { ffi::subset_font_has_glyph(self.0, value) }
    }

    pub fn style(&self) -> FontStyle {
        unsafe {
            FontStyle {
                italic: ffi::subset_font_is_italic(self.0),
                weight: ffi::subset_font_weight(self.0),
                width: ffi::subset_font_width(self.0),
            }
        }
    }

    pub fn upem(&self) -> u32 {
        unsafe { ffi::subset_font_upem(self.0) }
    }

    pub fn horizontal_extents(&self) -> Option<FontExtents> {
        extents(self.0, true)
    }

    pub fn vertical_extents(&self) -> Option<FontExtents> {
        extents(self.0, false)
    }
}

impl Clone for Font {
    fn clone(&self) -> Self {
        let raw = unsafe { ffi::subset_font_reference(self.0) };
        assert!(!raw.is_null());
        Self(raw)
    }
}

impl Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Font").finish_non_exhaustive()
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            ffi::subset_font_destroy(self.0);
        }
    }
}

pub fn find_best_font_match(
    fonts: &[Font],
    unicode_value: impl Into<UnicodeValue>,
    font_style: FontStyle,
) -> Option<usize> {
    struct Fonts<'a>(&'a [Font]);
    unsafe extern "C" fn font_provider(index: usize, context: *mut c_void) -> *mut ffi::SubsetFont {
        let fonts: &Fonts<'_> = unsafe { &*(context as *const Fonts<'_>) };
        (fonts.0)[index].0
    }
    let len = fonts.len();
    let fonts = Fonts(fonts);
    let mut best_index = 0usize;
    let found = unsafe {
        ffi::subset_find_best_font_match(
            unicode_value.into().into(),
            font_style,
            len,
            Some(font_provider),
            &fonts as *const _ as *mut _,
            &mut best_index as *mut _,
        )
    };
    if found {
        Some(best_index)
    } else {
        None
    }
}

fn extents(font: *mut ffi::SubsetFont, horizontal: bool) -> Option<FontExtents> {
    let mut ascender = 0i32;
    let mut descender = 0i32;
    let mut line_gap = 0i32;
    let found = unsafe {
        ffi::subset_font_extents(
            font,
            horizontal,
            &mut ascender as *mut _,
            &mut descender as *mut _,
            &mut line_gap as *mut _,
        )
    };
    if found {
        Some(FontExtents {
            ascender,
            descender,
            line_gap,
        })
    } else {
        None
    }
}
