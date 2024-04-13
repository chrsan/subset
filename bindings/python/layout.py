import itertools

from collections import deque
from collections.abc import Callable, Generator
from dataclasses import dataclass, field

from .lib import GlyphDrawer, Font, ShapeContext, find_best_font_match, text_runs  # type: ignore
from .types import FontRun, FontStyle, Glyph, GlyphRun, Path, PathVerb, Point


@dataclass(eq=False)
class Layout:
    fonts: list[Font] = field(repr=False)
    font_runs: list[FontRun] = field(default_factory=list)
    paragraph_base_level: int = 0
    unichars: list[int] = field(default_factory=list)

    def shape(
        self,
        embolden_strength: float = 0.02,
        slant: float = 0.25,
        emit_path_commands: bool = True,
    ) -> list[GlyphRun]:
        glyph_drawer = GlyphDrawer() if emit_path_commands else None
        shape_context = ShapeContext(self.unichars)
        glyph_runs = []
        for run in self.font_runs:
            font = self.fonts[run.font_index]
            if run.synthetic_bold or run.synthetic_slant:
                font = font.synthesize(
                    embolden_strength if run.synthetic_bold else None,
                    slant if run.synthetic_slant else None,
                )

            glyphs = []

            def glyph_callback(
                glyph_id: int,
                x_offset: int,
                y_offset: int,
                x_advance: int,
                y_advance: int,
            ):
                path = Path()

                def path_command_callback(
                    verb: int, points: list[float], coordinate_count: int
                ):
                    path.verbs.append(PathVerb(verb))
                    for i in range(0, coordinate_count, 2):
                        path.points.append(Point(x=points[i], y=points[i + 1]))

                if glyph_drawer is not None:
                    font.draw_glyph(glyph_id, glyph_drawer, path_command_callback)
                glyphs.append(
                    Glyph(
                        glyph_id=glyph_id,
                        x_offset=x_offset,
                        y_offset=y_offset,
                        x_advance=x_advance,
                        y_advance=y_advance,
                        path=path,
                    )
                )

            shape_context.shape(
                font,
                run.offset,
                run.length,
                run.bidi_level,
                run.script,
                glyph_callback,
            )
            glyph_runs.append(GlyphRun(font_run=run, glyphs=glyphs))
        return glyph_runs


def _split_run[
    T
](
    run_offset: int,
    run_length: int,
    last_value: T,
    callable: Callable[[int], T],
) -> Generator[tuple[int, int, T], None, None]:
    offset = run_offset
    length = 0
    for index in range(run_offset, run_offset + run_length):
        new_value = callable(index)
        if length != 0 and new_value != last_value:
            yield offset, length, last_value
            offset = index
            length = 0
        last_value = new_value
        length += 1
    yield offset, length, last_value


class LayoutBuilder:
    fonts: list[Font]
    unichars: list[int]
    font_styles: list[FontStyle]
    font_style_indices: list[int]

    def __init__(self, fonts: list[Font]) -> None:
        if not fonts:
            raise ValueError("empty fonts list")
        self.fonts = fonts
        self.unichars = []
        self.font_styles = []
        self.font_style_indices = []

    def append(self, text: str, font_style: FontStyle) -> None:
        start = len(self.unichars)
        self.unichars.extend([ord(c) for c in text])
        end = len(self.unichars)
        font_style_index = len(self.font_styles)
        self.font_styles.append(font_style)
        self.font_style_indices.extend(itertools.repeat(font_style_index, end - start))

    def has_missing_glyphs(self) -> bool:
        for unichar in self.unichars:
            for font in self.fonts:
                if font.has_glyph(unichar):
                    break
            else:
                return True
        return False

    def build(self) -> Layout:
        layout = Layout(fonts=self.fonts)
        if not self.unichars:
            return layout
        layout.unichars = self.unichars.copy()

        def text_run_callback(
            text_run_offset: int, text_run_length: int, bidi_level: int, script: int
        ):
            rtl = (bidi_level & 1) != 0
            font_runs = deque()
            for (
                font_style_run_offset,
                font_style_run_length,
                font_style,
            ) in _split_run(
                run_offset=text_run_offset,
                run_length=text_run_length,
                last_value=FontStyle(),
                callable=lambda index: self.font_styles[self.font_style_indices[index]],
            ):
                for offset, length, font_index in _split_run(
                    run_offset=font_style_run_offset,
                    run_length=font_style_run_length,
                    last_value=0,
                    callable=lambda index: find_best_font_match(
                        self.unichars[index],
                        font_style.italic,
                        font_style.weight,
                        font_style.width,
                        self.fonts,
                    )[1],
                ):
                    font = self.fonts[font_index]
                    synthetic_bold = font_style.weight > font.weight
                    synthetic_slant = font_style.italic and not font.is_italic
                    font_run = FontRun(
                        offset=offset,
                        length=length,
                        bidi_level=bidi_level,
                        script=script,
                        font_index=font_index,
                        font_style=font_style,
                        synthetic_bold=synthetic_bold,
                        synthetic_slant=synthetic_slant,
                    )
                    if rtl:
                        font_runs.appendleft(font_run)
                    else:
                        font_runs.append(font_run)
            layout.font_runs.extend(font_runs)

        layout.paragraph_base_level = text_runs(self.unichars, text_run_callback)
        return layout

    def clear(self) -> None:
        self.unichars.clear
        self.font_styles.clear()
        self.font_style_indices.clear()
