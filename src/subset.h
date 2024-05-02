#pragma once

#ifdef __cplusplus
#include <cstddef>
#include <cstdint>

extern "C" {
#else
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#endif

// NOLINTNEXTLINE
typedef struct SubsetGlyphDrawer SubsetGlyphDrawer;

SubsetGlyphDrawer* subset_glyph_drawer_create();

void subset_glyph_drawer_destroy(SubsetGlyphDrawer* drawer);

// NOLINTNEXTLINE
typedef struct SubsetFont SubsetFont;

SubsetFont* subset_font_create_from_data(const char* data, unsigned int length,
                                         unsigned int index);

SubsetFont* subset_font_create_from_file(const char* filename,
                                         unsigned int index);

SubsetFont* subset_font_reference(SubsetFont* font);

SubsetFont* subset_font_synthesize(SubsetFont* font,
                                   const float* embolden_strength,
                                   const float* slant);

void subset_font_destroy(SubsetFont* font);

bool subset_font_has_glyph(SubsetFont* font, uint32_t unichar);

bool subset_font_is_italic(SubsetFont* font);

float subset_font_weight(SubsetFont* font);

float subset_font_width(SubsetFont* font);

unsigned int subset_font_upem(SubsetFont* font);

bool subset_font_extents(SubsetFont* font, bool horizontal, int32_t* ascender,
                         int32_t* descender, int32_t* line_gap);

enum SubsetPathVerb {
  SUBSET_PATH_VERB_MOVE_TO = 0,
  SUBSET_PATH_VERB_LINE_TO = 1,
  SUBSET_PATH_VERB_QUAD_TO = 2,
  SUBSET_PATH_VERB_CUBIC_TO = 3,
  SUBSET_PATH_VERB_CLOSE = 4,
};

// NOLINTNEXTLINE
typedef void (*SubsetPathCommandCallback)(enum SubsetPathVerb verb,
                                          const float* points,
                                          size_t coordinate_count,
                                          void* context);

void subset_font_draw_glyph(SubsetFont* font, uint32_t glyph_id,
                            SubsetGlyphDrawer* drawer,
                            SubsetPathCommandCallback callback, void* context);

struct SubsetTextRun {
  size_t offset;
  size_t length;
  uint8_t bidi_level;
  uint32_t script;
};

// NOLINTNEXTLINE
typedef void (*SubsetTextRunCallback)(struct SubsetTextRun text_run,
                                      void* context);

int subset_text_runs(const uint32_t* unichars, size_t unichar_count,
                     uint8_t* paragraph_base_level,
                     SubsetTextRunCallback callback, void* context);

struct SubsetFontStyle {
  bool italic;
  float weight;
  float width;
};

// NOLINTNEXTLINE
typedef SubsetFont* (*SubsetFontProvider)(size_t index, void* context);

bool subset_find_best_font_match(uint32_t unichar,
                                 struct SubsetFontStyle font_style,
                                 size_t font_count,
                                 SubsetFontProvider font_provider,
                                 void* font_provider_context,
                                 size_t* best_index);

struct SubsetShapeParams {
  const uint32_t* unichars;
  size_t unichar_count;
  size_t offset;
  size_t length;
  uint8_t bidi_level;
  uint32_t script;
  const char* language;
};

struct SubsetGlyph {
  uint32_t glyph_id;
  int32_t x_offset;
  int32_t y_offset;
  int32_t x_advance;
  int32_t y_advance;
};

// NOLINTNEXTLINE
typedef void (*SubsetShapeCallback)(struct SubsetGlyph glyph, void* context);

bool subset_shape(SubsetFont* font, const struct SubsetShapeParams* params,
                  SubsetShapeCallback callback, void* context);

#ifdef __cplusplus
}
#endif
