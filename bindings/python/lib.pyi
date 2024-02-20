from collections.abc import Callable
from pathlib import Path
from typing import NamedTuple

class GlyphDrawer:
    def __init__(self) -> None: ...

class FontExtents(NamedTuple):
    ascender: int
    descender: int
    line_gap: int

class Font:
    def has_glyph(self, uchar: int) -> bool: ...
    @property
    def is_italic(self) -> bool: ...
    @property
    def weight(self) -> float: ...
    @property
    def width(self) -> float: ...
    @property
    def upem(self) -> int: ...
    def extents(self, horizontal: bool = True) -> FontExtents: ...
    def clone(self) -> Font: ...
    def scale(self, font_size: float) -> float: ...
    def synthesize(
        self, embolden_strength: float | None, slant: float | None
    ) -> Font: ...
    def draw_glyph(
        self,
        glyph_id: int,
        glyph_drawer: GlyphDrawer,
        callback: Callable[[int, list[float]], None],
    ) -> None: ...
    @classmethod
    def from_data(cls, data: bytes, index: int = 0) -> Font: ...
    @classmethod
    def from_file_path(cls, filename: str | Path, index: int = 0) -> Font: ...

def text_runs(
    unichars: list[int], callback: Callable[[int, int, int, int], None]
) -> int: ...
def find_best_font_match(
    unichar: int, italic: bool, weight: float, width: float, fonts: list[Font]
) -> tuple[bool, int]: ...

class ShapeContext:
    def __init__(self, unichars: list[int]) -> None: ...
    def shape(
        self,
        font: Font,
        offset: int,
        length: int,
        bidi_level: int,
        script: int,
        callback: Callable[[int, int, int, int, int], None],
    ) -> bool: ...
