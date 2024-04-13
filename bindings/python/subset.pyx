# cython: language_level=3

import os

from collections import namedtuple
from collections.abc import Callable
from pathlib import Path

from cpython.mem cimport PyMem_Malloc, PyMem_Free
from libc.stddef cimport size_t
from libc.stdint cimport int32_t, uint8_t, uint32_t

cdef extern from "deps/lib/subset.h":
    ctypedef struct SubsetGlyphDrawer:
        pass

    cdef SubsetGlyphDrawer* subset_glyph_drawer_create()

    cdef void subset_glyph_drawer_destroy(SubsetGlyphDrawer* drawer)

    cdef enum SubsetPathVerb:
        SUBSET_PATH_VERB_MOVE_TO = 0
        SUBSET_PATH_VERB_LINE_TO = 1
        SUBSET_PATH_VERB_QUAD_TO = 2
        SUBSET_PATH_VERB_CUBIC_TO = 3
        SUBSET_PATH_VERB_CLOSE = 4

    ctypedef void (*SubsetPathCommandCallback)(SubsetPathVerb verb,
                                               const float* points,
                                               size_t coordinate_count,
                                               void* context)

    ctypedef struct SubsetFont:
        pass

    cdef SubsetFont* subset_font_create_from_data(const char* data,
                                                  unsigned int length,
                                                  unsigned int index)

    cdef SubsetFont* subset_font_create_from_file(const char* filename,
                                                  unsigned int index)

    cdef bint subset_font_has_glyph(SubsetFont* font, uint32_t unichar)

    cdef void subset_font_destroy(SubsetFont* font)

    cdef bint subset_font_is_italic(SubsetFont* font)

    cdef float subset_font_weight(SubsetFont* font)

    cdef float subset_font_width(SubsetFont* font)

    cdef unsigned int subset_font_upem(SubsetFont* font)

    cdef bint subset_font_extents(SubsetFont* font,
                                  bint horizontal,
                                  int32_t* ascender,
                                  int32_t* descender,
                                  int32_t* line_gap)

    cdef SubsetFont* subset_font_reference(SubsetFont* font)

    cdef SubsetFont* subset_font_synthesize(SubsetFont* font,
                                            const float* embolden_strength,
                                            const float* slant)

    cdef void subset_font_draw_glyph(SubsetFont* font,
                                     uint32_t glyph_id,
                                     SubsetGlyphDrawer* drawer,
                                     SubsetPathCommandCallback callback,
                                     void* context)

    ctypedef void (*SubsetTextRunCallback)(SubsetTextRun text_run,
                                           void* context)
    cdef struct SubsetTextRun:
        size_t offset
        size_t length
        uint8_t bidi_level
        uint32_t script

    cdef int subset_text_runs(const uint32_t* unichars,
                              size_t unichar_count,
                              uint8_t* paragraph_base_level,
                              SubsetTextRunCallback callback,
                              void* context);

    cdef struct SubsetFontStyle:
        bint italic
        float weight;
        float width;

    ctypedef SubsetFont* (*SubsetFontProvider)(size_t index, void* context)

    cdef bint subset_find_best_font_match(uint32_t unichar,
                                          SubsetFontStyle font_style,
                                          size_t font_count,
                                          SubsetFontProvider font_provider,
                                          void* font_provider_context,
                                          size_t* best_index)

    cdef struct SubsetShapeParams:
        const uint32_t* unichars
        size_t unichar_count
        size_t offset
        size_t length
        uint8_t bidi_level
        uint32_t script
        const char* language

    cdef struct SubsetGlyph:
        uint32_t glyph_id
        int32_t x_offset
        int32_t y_offset
        int32_t x_advance
        int32_t y_advance

    ctypedef void (*SubsetShapeCallback)(SubsetGlyph glyph, void* context)

    cdef bint subset_shape(SubsetFont* font,
                           const SubsetShapeParams* params,
                           SubsetShapeCallback callback,
                           void* context)



cdef class GlyphDrawer:
    cdef SubsetGlyphDrawer* _glyph_drawer;
    
    def __cinit__(self) -> None:
        self._glyph_drawer = subset_glyph_drawer_create()
        if self._glyph_drawer is NULL:
            raise MemoryError()

    def __dealloc__(self) -> None:
        if self._glyph_drawer is not NULL:
            subset_glyph_drawer_destroy(self._glyph_drawer)
            self._glyph_drawer = NULL


cdef void _draw_glyph_callback(SubsetPathVerb verb, const float* points, size_t coordinate_count, void* context) noexcept:
    cdef list pts = []
    for i in range(0, coordinate_count):
        pts.append(points[i])
    (<object>context)(verb, pts, coordinate_count)
    

FontExtents = namedtuple("FontExtents", ["ascender", "descender", "line_gap"])


cdef class Font:
    cdef SubsetFont* _font

    def __dealloc__(self) -> None:
        if self._font is not NULL:
            subset_font_destroy(self._font)
        self._font = NULL

    def has_glyph(self, uchar: int) -> bool:
        return subset_font_has_glyph(self._font, uchar)

    @property
    def is_italic(self) -> bool:
        return subset_font_is_italic(self._font)

    @property
    def weight(self) -> float:
        return subset_font_weight(self._font)

    @property
    def width(self) -> float:
        return subset_font_width(self._font)

    @property
    def upem(self) -> int:
        return subset_font_upem(self._font)

    def extents(self, horizontal: bool = True) -> FontExtents:
        cdef int32_t ascender
        cdef int32_t descender
        cdef int32_t line_gap
        subset_font_extents(self._font, horizontal, &ascender, &descender, &line_gap)
        return FontExtents(ascender, descender, line_gap)

    def clone(self) -> Font:
        cdef SubsetFont* clone = subset_font_reference(self._font)
        if clone is NULL:
            raise MemoryError()
        cdef Font font = Font.__new__(Font)
        font._font = clone
        return font

    def scale(self, font_size: float) -> float:
        return font_size / self.upem

    def synthesize(self, embolden_strength: float | None, slant: float | None) -> Font:
        if embolden_strength is None and slant is None:
            return self
        cdef float value1
        cdef float* ptr1 = NULL
        if embolden_strength is not None:
            value1 = embolden_strength
            ptr1 = &value1
        cdef float value2
        cdef float* ptr2 = NULL
        if slant is not None:
            value2 = slant
            ptr2 = &value2
        cdef SubsetFont* synthesized_font = subset_font_synthesize(self._font, ptr1, ptr2)
        if synthesized_font is NULL:
            raise MemoryError()
        cdef Font font = Font.__new__(Font)
        font._font = synthesized_font
        return font

    def draw_glyph(self, glyph_id: int, glyph_drawer: GlyphDrawer, callback: Callable[[int, list[float], int], None]) -> None:
        subset_font_draw_glyph(self._font, glyph_id, glyph_drawer._glyph_drawer, _draw_glyph_callback, <void*>callback)

    @classmethod
    def from_data(cls, data: bytes, index: int = 0) -> Font:
        cdef SubsetFont* font = subset_font_create_from_data(data, len(data), index)
        if font is NULL:
            raise MemoryError()
        cdef Font instance = cls(None)
        instance._font = font
        return instance

    @classmethod
    def from_file_path(cls, filename: str | Path, index: int = 0) -> Font:
        cdef bytes packed = os.fsencode(filename)
        cdef SubsetFont* font = subset_font_create_from_file(<char*>packed, index)
        if font is NULL:
            raise MemoryError()
        cdef Font instance = cls(None)
        instance._font = font
        return instance


cdef void _text_run_callback(SubsetTextRun text_run, void* context) noexcept:
    (<object>context)(text_run)

def text_runs(unichars: list[int], callback: Callable[[int, int, int, int], None]) -> int:
    if not unichars:
        return 0
    cdef uint32_t* uc = <uint32_t*>PyMem_Malloc(len(unichars) * sizeof(uint32_t))
    if uc is NULL:
        raise MemoryError()
    for i in range(len(unichars)):
        uc[i] = unichars[i]

    def callback_delegate(text_run):
        callback(text_run["offset"], text_run["length"], text_run["bidi_level"], text_run["script"])

    cdef uint8_t paragraph_base_level = 0
    try:
        rv = subset_text_runs(uc, len(unichars), &paragraph_base_level, _text_run_callback, <void*>callback_delegate)
        if rv != 0:
            raise ValueError()
    finally:
        PyMem_Free(uc)
    return paragraph_base_level

cdef SubsetFont* _font_provider(size_t index, void* context) noexcept:
    cdef Font font = (<list>context)[index]
    return font._font

def find_best_font_match(unichar: int, italic: bool, weight: float, width: float, fonts: list[Font]) -> tuple[bool, int]:
    cdef SubsetFontStyle fs = SubsetFontStyle(
        italic=italic,
        weight=weight,
        width=width,
    )
    cdef size_t best_index = 0;
    cdef bint ok = subset_find_best_font_match(unichar, fs, len(fonts), _font_provider, <void*>fonts, &best_index)
    return ok, best_index

cdef void _shape_callback(SubsetGlyph glyph, void* context) noexcept:
    (<object>context)(glyph)


cdef class ShapeContext:
    cdef uint32_t* _unichars
    cdef size_t _unichar_count

    def __cinit__(self, unichars: list[int]) -> None:
        cdef uint32_t* uc = <uint32_t*>PyMem_Malloc(len(unichars) * sizeof(uint32_t))
        if uc is NULL:
            raise MemoryError()
        for i in range(len(unichars)):
            uc[i] = unichars[i]
        self._unichars = uc
        self._unichar_count = len(unichars)

    def __dealloc__(self) -> None:
        PyMem_Free(self._unichars)

    # TODO: Language param
    def shape(self, font: Font, offset: int, length: int, bidi_level: int, script: int, callback: Callable[[int, int, int, int, int], None]) -> bool:
        cdef SubsetShapeParams params = SubsetShapeParams(
            unichars=self._unichars,
            unichar_count=self._unichar_count,
            offset=offset,
            length=length,
            bidi_level=bidi_level,
            script=script,
            language=NULL,
        )

        def callback_delegate(glyph):
            callback(glyph["glyph_id"], glyph["x_offset"], glyph["y_offset"], glyph["x_advance"], glyph["y_advance"])

        return subset_shape(font._font, &params, _shape_callback, <void*>callback_delegate)
