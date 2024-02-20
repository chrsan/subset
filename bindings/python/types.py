from dataclasses import dataclass, field
from enum import IntEnum
from typing import NamedTuple


class FontStyle(NamedTuple):
    italic: bool = False
    weight: float = 400.0
    width: float = 100.0


class FontRun(NamedTuple):
    offset: int
    length: int
    bidi_level: int
    script: int
    font_index: int
    font_style: FontStyle
    synthetic_bold: bool
    synthetic_slant: bool


class Point(NamedTuple):
    x: float
    y: float


@dataclass(frozen=True)
class Transform:
    sx: float = 1.0
    sy: float = 1.0
    tx: float = 0.0
    ty: float = 0.0

    @staticmethod
    def translate(tx: float, ty: float) -> "Transform":
        return Transform(tx=tx, ty=ty)

    @staticmethod
    def scale(sx: float, sy: float) -> "Transform":
        return Transform(sx=sx, sy=sy)

    def combine(self, other: "Transform") -> "Transform":
        sx = self.sx * other.sx
        sy = self.sy * other.sy
        tx = self.tx * other.sx + other.tx
        ty = self.ty * other.sy + other.ty
        return Transform(sx=sx, sy=sy, tx=tx, ty=ty)

    def pre_translate(self, tx: float, ty: float) -> "Transform":
        return Transform.translate(tx=tx, ty=ty).combine(self)

    def post_translate(self, tx: float, ty: float) -> "Transform":
        return self.combine(Transform.translate(tx=tx, ty=ty))

    def pre_scale(self, sx: float, sy: float) -> "Transform":
        return Transform.scale(sx=sx, sy=sy).combine(self)

    def post_scale(self, sx: float, sy: float) -> "Transform":
        return self.combine(Transform.scale(sx=sx, sy=sy))

    def transform_point(self, point: Point) -> Point:
        return Point(x=point.x * self.sx + self.tx, y=point.y * self.sy + self.ty)

    def transform_points(self, points: list[Point]) -> None:
        for i in range(0, len(points)):
            points[i] = self.transform_point(points[i])


class PathVerb(IntEnum):
    MOVE_TO = 0
    LINE_TO = 1
    QUAD_TO = 2
    CUBIC_TO = 3
    CLOSE_PATH = 4

    def num_points(self) -> int:
        match self:
            case PathVerb.MOVE_TO | PathVerb.LINE_TO:
                return 1
            case PathVerb.QUAD_TO:
                return 2
            case PathVerb.CUBIC_TO:
                return 3
            case _:
                return 0


@dataclass(eq=False)
class Path:
    verbs: list[PathVerb] = field(default_factory=list)
    points: list[Point] = field(default_factory=list)


class Glyph(NamedTuple):
    glyph_id: int
    x_offset: int
    y_offset: int
    x_advance: int
    y_advance: int
    path: Path


class GlyphRun(NamedTuple):
    font_run: FontRun
    glyphs: list[Glyph]
