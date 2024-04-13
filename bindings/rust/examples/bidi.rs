use std::path::Path;

use anyhow::{anyhow, Result};
use subset::{Font, FontStyle, GlyphRun, LayoutBuilder, PathCommand, ShapeParams};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

fn main() -> Result<()> {
    let fonts = load_fonts()?;
    let scale = 72.0 / fonts[0].upem() as f32;
    let mut builder = LayoutBuilder::new(&fonts);
    builder.push(
        "داستان SVG Tiny 1.2 طولا ني است.".chars(),
        FontStyle::default(),
    );
    let runs = builder.build().shape(ShapeParams {
        emit_path_commands: true,
        ..Default::default()
    });
    let mut pixmap = Pixmap::new(1280, 120).unwrap();
    pixmap.fill(Color::WHITE);
    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    let transform = Transform::from_translate(100.0, 75.0).pre_scale(scale, -scale);
    let mut cx = 0.0;
    let mut cy = 0.0;
    for run in runs {
        draw(&mut pixmap, &paint, &transform, run, &mut cx, &mut cy);
    }
    pixmap.save_png("bidi.png")?;
    Ok(())
}

fn load_font(path: impl AsRef<Path>) -> Result<Font> {
    let path = path.as_ref();
    Font::from_file(path, 0).ok_or_else(|| anyhow!("could not load {}", path.display()))
}

fn load_fonts() -> Result<Vec<Font>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Ok(vec![
        load_font(manifest_dir.join("../../fonts/NotoSans-Regular.ttf"))?,
        load_font(manifest_dir.join("../../fonts/NotoSansArabic-Regular.ttf"))?,
    ])
}

fn add_path(
    builder: &mut PathBuilder,
    transform: &Transform,
    iter: impl Iterator<Item = PathCommand>,
) {
    for command in iter {
        match command {
            PathCommand::MoveTo(point) => {
                let mut point = point.into();
                transform.map_point(&mut point);
                builder.move_to(point.x, point.y);
            }
            PathCommand::LineTo(point) => {
                let mut point = point.into();
                transform.map_point(&mut point);
                builder.line_to(point.x, point.y);
            }
            PathCommand::QuadTo(point1, point2) => {
                let mut point1 = point1.into();
                transform.map_point(&mut point1);
                let mut point2 = point2.into();
                transform.map_point(&mut point2);
                builder.quad_to(point1.x, point1.y, point2.x, point2.y);
            }
            PathCommand::CubicTo(point1, point2, point3) => {
                let mut point1 = point1.into();
                transform.map_point(&mut point1);
                let mut point2 = point2.into();
                transform.map_point(&mut point2);
                let mut point3 = point3.into();
                transform.map_point(&mut point3);
                builder.cubic_to(point1.x, point1.y, point2.x, point2.y, point3.x, point3.y);
            }
            PathCommand::Close => {
                builder.close();
            }
        }
    }
}

fn draw(
    pixmap: &mut Pixmap,
    paint: &Paint,
    transform: &Transform,
    run: GlyphRun,
    cx: &mut f32,
    cy: &mut f32,
) {
    for (glyph, path) in run.glyphs.into_iter().zip(run.paths.into_iter()) {
        let transform =
            transform.pre_translate(*cx + glyph.x_offset as f32, *cy + glyph.y_offset as f32);
        let mut builder = PathBuilder::new();
        add_path(&mut builder, &transform, path.iter());
        if let Some(path) = builder.finish() {
            pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
        }
        *cx += glyph.x_advance as f32;
        *cy += glyph.y_advance as f32;
    }
}
