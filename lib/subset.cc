#include <hb.h>

#include <array>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <vector>

extern "C" {
#include <SheenBidi.h>
}

#include "subset.h"

namespace {
hb_font_t* CreateFont(hb_blob_t* blob, unsigned int index) noexcept {
  if (blob == nullptr) {
    return nullptr;
  }

  auto* face = hb_face_create(blob, index);
  hb_blob_destroy(blob);
  if (face == nullptr) {
    return nullptr;
  }

  if (face == hb_face_get_empty()) {
    hb_face_destroy(face);
    return nullptr;
  }

  auto* font = hb_font_create(face);
  hb_face_destroy(face);
  if (font == nullptr) {
    return nullptr;
  }

  if (font == hb_font_get_empty()) {
    hb_font_destroy(font);
    return nullptr;
  }

  return font;
}

struct PathContext {
  SubsetPathCommandCallback callback;
  void* callback_context;

  std::array<float, 6> points{};  // NOLINT

  PathContext(SubsetPathCommandCallback callback, void* callback_context)
      : callback(callback), callback_context(callback_context) {}
};

void MoveTo(hb_draw_funcs_t* /* draw_funcs */, void* draw_data,
            hb_draw_state_t* /* draw_state */, float to_x, float to_y,
            void* /* user_data */) noexcept {
  auto& path_context = *static_cast<PathContext*>(draw_data);
  path_context.points[0] = to_x;
  path_context.points[1] = to_y;
  path_context.callback(SUBSET_PATH_VERB_MOVE_TO, path_context.points.data(), 2,
                        path_context.callback_context);
}

void LineTo(hb_draw_funcs_t* /* draw_funcs */, void* draw_data,
            hb_draw_state_t* /* draw_state */, float to_x, float to_y,
            void* /* user_data */) noexcept {
  auto& path_context = *static_cast<PathContext*>(draw_data);
  path_context.points[0] = to_x;
  path_context.points[1] = to_y;
  path_context.callback(SUBSET_PATH_VERB_LINE_TO, path_context.points.data(), 2,
                        path_context.callback_context);
}

void QuadTo(hb_draw_funcs_t* /* draw_funcs */, void* draw_data,
            hb_draw_state_t* /* draw_state */, float control_x, float control_y,
            float to_x, float to_y, void* /* user_data */) noexcept {
  auto& path_context = *static_cast<PathContext*>(draw_data);
  path_context.points[0] = control_x;
  path_context.points[1] = control_y;
  path_context.points[2] = to_x;
  path_context.points[3] = to_y;
  path_context.callback(SUBSET_PATH_VERB_QUAD_TO, path_context.points.data(), 4,
                        path_context.callback_context);
}

void CubicTo(hb_draw_funcs_t* /* draw_funcs */, void* draw_data,
             hb_draw_state_t* /* draw_state */, float control1_x,
             float control1_y, float control2_x, float control2_y, float to_x,
             float to_y, void* /* user_data */) noexcept {
  auto& path_context = *static_cast<PathContext*>(draw_data);
  path_context.points[0] = control1_x;
  path_context.points[1] = control1_y;
  path_context.points[2] = control2_x;
  path_context.points[3] = control2_y;
  path_context.points[4] = to_x;
  path_context.points[5] = to_y;
  path_context.callback(SUBSET_PATH_VERB_CUBIC_TO, path_context.points.data(),
                        6, path_context.callback_context);
}

void ClosePath(hb_draw_funcs_t* /* draw_funcs */, void* draw_data,
               hb_draw_state_t* /* draw_state */,
               void* /* user_data */) noexcept {
  auto& path_context = *static_cast<PathContext*>(draw_data);
  path_context.callback(SUBSET_PATH_VERB_CLOSE, nullptr, 0,
                        path_context.callback_context);
}

struct BidiAlgorithmDeleter {
  void operator()(SBAlgorithmRef algorithm) noexcept {
    SBAlgorithmRelease(algorithm);
  }
};

struct BidiParagraphDeleter {
  void operator()(SBParagraphRef paragraph) noexcept {
    SBParagraphRelease(paragraph);
  }
};

struct BidiLineDeleter {
  void operator()(SBLineRef line) noexcept { SBLineRelease(line); }
};

void ScriptsForRun(const uint32_t* unichars, const SBRun& run,
                   std::vector<hb_script_t>& scripts) noexcept {
  auto* unicode_funcs = hb_unicode_funcs_get_default();

  scripts.clear();
  scripts.reserve(run.length);

  bool backwards_scan{false};
  hb_script_t last_script{HB_SCRIPT_INVALID};
  for (std::size_t index = 0; index < run.length; ++index) {
    auto script =
        hb_unicode_script(unicode_funcs, unichars[run.offset + index]);
    scripts.push_back(script);
    if (script == HB_SCRIPT_COMMON || script == HB_SCRIPT_INHERITED) {
      if (last_script != HB_SCRIPT_INVALID) {
        scripts.back() = last_script;
      } else {
        backwards_scan = true;
      }
    } else {
      last_script = script;
    }
  }

  if (backwards_scan) {
    last_script = HB_SCRIPT_INVALID;
    for (auto it = scripts.rbegin(); it != scripts.rend(); ++it) {
      auto script = *it;
      if (script == HB_SCRIPT_COMMON || script == HB_SCRIPT_INHERITED) {
        if (last_script != HB_SCRIPT_INVALID) {
          *it = last_script;
        }
      } else {
        last_script = script;
      }
    }
  }
}
}  // namespace

#define DRAWER(drawer) reinterpret_cast<hb_draw_funcs_t*>(drawer)  // NOLINT

SubsetGlyphDrawer* subset_glyph_drawer_create() {
  auto* draw_funcs = hb_draw_funcs_create();
  hb_draw_funcs_set_move_to_func(draw_funcs, MoveTo, nullptr, nullptr);
  hb_draw_funcs_set_line_to_func(draw_funcs, LineTo, nullptr, nullptr);
  hb_draw_funcs_set_quadratic_to_func(draw_funcs, QuadTo, nullptr, nullptr);
  hb_draw_funcs_set_cubic_to_func(draw_funcs, CubicTo, nullptr, nullptr);
  hb_draw_funcs_set_close_path_func(draw_funcs, ClosePath, nullptr, nullptr);
  hb_draw_funcs_make_immutable(draw_funcs);
  return reinterpret_cast<SubsetGlyphDrawer*>(draw_funcs);
}

void subset_glyph_drawer_destroy(SubsetGlyphDrawer* drawer) {
  if (drawer != nullptr) {
    hb_draw_funcs_destroy(DRAWER(drawer));
  }
}

#define FONT(font) reinterpret_cast<hb_font_t*>(font)  // NOLINT

extern "C" {
SubsetFont* subset_font_create_from_data(const char* data, unsigned int length,
                                         unsigned int index) {
  auto* blob = hb_blob_create_or_fail(data, length, HB_MEMORY_MODE_DUPLICATE,
                                      nullptr, nullptr);
  return reinterpret_cast<SubsetFont*>(CreateFont(blob, index));
}

SubsetFont* subset_font_create_from_file(const char* filename,
                                         unsigned int index) {
  auto* blob = hb_blob_create_from_file_or_fail(filename);
  return reinterpret_cast<SubsetFont*>(CreateFont(blob, index));
}

SubsetFont* subset_font_reference(SubsetFont* font) {
  if (font == nullptr) {
    return nullptr;
  }

  return reinterpret_cast<SubsetFont*>(hb_font_reference(FONT(font)));
}

SubsetFont* subset_font_synthesize(SubsetFont* font,
                                   const float* embolden_strength,
                                   const float* slant) {
  if (font == nullptr) {
    return nullptr;
  }

  if (embolden_strength == nullptr && slant == nullptr) {
    return font;
  }

  // N.B. When using a sub font embolden doesn't seem to work.
  auto* face = hb_font_get_face(FONT(font));
  auto* new_font = hb_font_create(face);
  if (embolden_strength != nullptr) {
    hb_font_set_synthetic_bold(new_font, *embolden_strength, *embolden_strength,
                               0);
  }

  if (slant != nullptr) {
    hb_font_set_synthetic_slant(new_font, *slant);
  }

  return reinterpret_cast<SubsetFont*>(new_font);
}

void subset_font_destroy(SubsetFont* font) {
  if (font != nullptr) {
    hb_font_destroy(FONT(font));
  }
}

bool subset_font_has_glyph(SubsetFont* font, uint32_t unichar) {
  hb_codepoint_t glyph{0};
  return hb_font_get_nominal_glyph(FONT(font), unichar, &glyph) != 0;
}

bool subset_font_is_italic(SubsetFont* font) {
  return hb_style_get_value(FONT(font), HB_STYLE_TAG_ITALIC) == 1.0;
}

float subset_font_weight(SubsetFont* font) {
  return hb_style_get_value(FONT(font), HB_STYLE_TAG_WEIGHT);
}

float subset_font_width(SubsetFont* font) {
  return hb_style_get_value(FONT(font), HB_STYLE_TAG_WIDTH);
}

unsigned int subset_font_upem(SubsetFont* font) {
  auto* face = hb_font_get_face(FONT(font));
  return hb_face_get_upem(face);
}

bool subset_font_extents(SubsetFont* font, bool horizontal, int32_t* ascender,
                         int32_t* descender, int32_t* line_gap) {
  hb_font_extents_t extents;
  auto found = horizontal ? hb_font_get_h_extents(FONT(font), &extents)
                          : hb_font_get_v_extents(FONT(font), &extents);
  if (found != 0) {
    if (ascender != nullptr) {
      *ascender = extents.ascender;
    }

    if (descender != nullptr) {
      *descender = extents.descender;
    }

    if (line_gap != nullptr) {
      *line_gap = extents.line_gap;
    }
  }

  return found != 0;
}

void subset_font_draw_glyph(SubsetFont* font, uint32_t glyph_id,
                            SubsetGlyphDrawer* drawer,
                            SubsetPathCommandCallback callback, void* context) {
  if (font == nullptr || drawer == nullptr || callback == nullptr) {
    return;
  }

  PathContext path_context{callback, context};
  hb_font_draw_glyph(FONT(font), glyph_id, DRAWER(drawer), &path_context);
}

int subset_text_runs(const uint32_t* unichars, size_t unichar_count,
                     uint8_t* paragraph_base_level,
                     SubsetTextRunCallback callback, void* context) {
  if (unichars == nullptr || unichar_count == 0) {
    return 0;
  }

  SBCodepointSequence codepoint_seq{SBStringEncodingUTF32, (void*)unichars,
                                    unichar_count};
  std::unique_ptr<_SBAlgorithm, BidiAlgorithmDeleter> algo(
      SBAlgorithmCreate(&codepoint_seq), BidiAlgorithmDeleter{});
  if (!algo) {
    return 1;
  }

  std::unique_ptr<_SBParagraph, BidiParagraphDeleter> para(
      SBAlgorithmCreateParagraph(algo.get(), 0, INT32_MAX, SBLevelDefaultLTR),
      BidiParagraphDeleter{});
  if (!para) {
    return 1;
  }

  if (SBParagraphGetLength(para.get()) != unichar_count) {
    return 2;
  }

  if (paragraph_base_level != nullptr) {
    *paragraph_base_level = SBParagraphGetBaseLevel(para.get());
  }

  if (callback == nullptr) {
    return 0;
  }

  std::unique_ptr<_SBLine, BidiLineDeleter> line(
      SBParagraphCreateLine(para.get(), 0, unichar_count), BidiLineDeleter{});
  if (!line) {
    return 1;
  }

  auto run_count = SBLineGetRunCount(line.get());
  const auto* runs = SBLineGetRunsPtr(line.get());

  std::vector<hb_script_t> scripts{};
  for (std::size_t run_index = 0; run_index < run_count; ++run_index) {
    auto run = runs[run_index];
    ScriptsForRun(unichars, run, scripts);

    // Split the BiDi run on script boundaries if needed.
    std::size_t offset = 0;
    std::size_t remaining = run.length;
    hb_script_t last_script = HB_SCRIPT_INVALID;
    for (std::size_t index = 0; index < run.length; ++index) {
      auto script = scripts[index];
      if (last_script != HB_SCRIPT_INVALID && script != last_script) {
        auto len = index - offset;
        SubsetTextRun text_run{
            .offset = run.offset + offset,
            .length = len,
            .bidi_level = run.level,
            .script = script,
        };
        callback(text_run, context);
        offset = index;
        remaining -= len;
      }

      last_script = script;
    }

    SubsetTextRun text_run{
        .offset = run.offset + offset,
        .length = remaining,
        .bidi_level = run.level,
        .script = last_script,
    };
    callback(text_run, context);
  }

  return 0;
}

// NOLINTNEXTLINE
bool subset_find_best_font_match(uint32_t unichar, SubsetFontStyle font_style,
                                 size_t font_count,
                                 SubsetFontProvider font_provider,
                                 void* font_provider_context,
                                 size_t* best_index) {
  if (font_provider == nullptr || font_count == 0) {
    return false;
  }

  float max_score{0};
  size_t max_index{0};
  for (size_t index = 0; index < font_count; ++index) {
    auto* font = font_provider(index, font_provider_context);
    if (!subset_font_has_glyph(font, unichar)) {
      continue;
    }

    constexpr float kMaxWidthScore = 225.0;
    float width_score{0};
    auto width = subset_font_width(font);
    if (font_style.width <= 100.0) {
      if (width <= font_style.width) {
        width_score = kMaxWidthScore - font_style.width + width;
      } else {
        width_score = kMaxWidthScore - width;
      }
    } else {
      if (width > font_style.width) {
        width_score = kMaxWidthScore + font_style.width - width;
      } else {
        width_score = width;
      }
    }

    constexpr float kItalicMatchScore = 3.0;
    float italic_score{1};
    if (subset_font_is_italic(font)) {
      if (font_style.italic) {
        italic_score = kItalicMatchScore;
      }
    } else {
      if (!font_style.italic) {
        italic_score = kItalicMatchScore;
      }
    }

    constexpr float kMaxWeightScore = 1000.0;
    constexpr float kNormalWeight = 400.0;
    constexpr float kMediumWeight = 500.0;
    float weight_score{0};
    auto weight = subset_font_weight(font);
    if (font_style.weight == weight) {
      weight_score = kMaxWeightScore;
    } else if (font_style.weight < kNormalWeight) {
      if (weight <= font_style.weight) {
        weight_score = kMaxWeightScore - font_style.weight + weight;
      } else {
        weight_score = kMaxWeightScore - weight;
      }
    } else if (font_style.weight <= kMediumWeight) {
      if (weight >= font_style.weight && weight <= kMediumWeight) {
        weight_score = kMaxWeightScore + font_style.weight - weight;
      } else if (weight <= font_style.weight) {
        weight_score = kMediumWeight + weight;
      } else {
        weight_score = kMaxWeightScore - weight;
      }
    } else if (font_style.weight > kMediumWeight) {
      if (weight > font_style.weight) {
        weight_score = kMaxWeightScore + font_style.weight - weight;
      } else {
        weight_score = weight;
      }
    }

    constexpr float kWidthScoreMultiplier = 1e7;
    constexpr float kItalicScoreMultiplier = 1e4;
    float score = width_score * kWidthScoreMultiplier +
                  italic_score * kItalicScoreMultiplier + weight_score;
    if (max_score < score) {
      max_score = score;
      max_index = index;
    }
  }

  if (max_score == 0.0) {
    return false;
  }

  if (best_index != nullptr) {
    *best_index = max_index;
  }

  return true;
}

bool subset_shape(SubsetFont* font, const SubsetShapeParams* params,
                  SubsetShapeCallback callback, void* context) {
  if (font == nullptr || params == nullptr || callback == nullptr) {
    return false;
  }

  constexpr auto kIntMax = std::numeric_limits<int>::max();
  if (params->unichar_count > kIntMax || params->length > kIntMax) {
    return false;
  }

  auto* buf = hb_buffer_create();
  hb_buffer_add_utf32(buf, params->unichars,
                      static_cast<int>(params->unichar_count), params->offset,
                      static_cast<int>(params->length));
  hb_buffer_set_direction(buf, (params->bidi_level & 1U) != 0
                                   ? HB_DIRECTION_RTL
                                   : HB_DIRECTION_LTR);
  hb_buffer_set_script(buf, static_cast<hb_script_t>(params->script));
  if (params->language == nullptr) {
    hb_buffer_set_language(buf, hb_language_get_default());
  } else {
    const auto* lang = hb_language_from_string(params->language, -1);
    if (lang == HB_LANGUAGE_INVALID) {
      // TODO(chrsan): Should we return `false` here instead?
      hb_buffer_set_language(buf, hb_language_get_default());
    } else {
      hb_buffer_set_language(buf, lang);
    }
  }

  hb_shape(FONT(font), buf, nullptr, 0);

  unsigned int glyph_count{0};
  auto* glyph_info = hb_buffer_get_glyph_infos(buf, &glyph_count);
  auto* glyph_pos = hb_buffer_get_glyph_positions(buf, &glyph_count);
  for (unsigned int index = 0; index < glyph_count; ++index) {
    SubsetGlyph glyph{
        .glyph_id = glyph_info[index].codepoint,
        .x_offset = glyph_pos[index].x_offset,
        .y_offset = glyph_pos[index].y_offset,
        .x_advance = glyph_pos[index].x_advance,
        .y_advance = glyph_pos[index].y_advance,
    };
    callback(glyph, context);
  }

  hb_buffer_destroy(buf);
  return true;
}
}
