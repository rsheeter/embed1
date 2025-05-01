use color::{AlphaColor, DynamicColor, Srgb};
use kurbo::{BezPath, PathEl};
use png::EncodingError;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

pub mod draw;

trait ToPixmapColor {
    fn to_pixmap_color(&self) -> Color;
}

impl ToPixmapColor for DynamicColor {
    fn to_pixmap_color(&self) -> Color {
        let srgba: AlphaColor<Srgb> = self.to_alpha_color();
        let [r, g, b, a] = srgba.components;
        Color::from_rgba(r, g, b, a).expect("Color")
    }
}

pub fn draw_png(
    pixmap: &mut Pixmap,
    fill: DynamicColor,
    backdrop: DynamicColor,
    path: BezPath,
) -> Result<Vec<u8>, EncodingError> {
    // https://github.com/linebender/tiny-skia/blob/main/examples/fill.rs basically
    pixmap.fill(backdrop.to_pixmap_color());

    let mut paint = Paint::default();
    paint.set_color(fill.to_pixmap_color());
    let path = {
        let mut pb = PathBuilder::new();
        for el in path {
            match el {
                PathEl::MoveTo(p) => pb.move_to(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => pb.line_to(p.x as f32, p.y as f32),
                PathEl::QuadTo(c0, p) => {
                    pb.quad_to(c0.x as f32, c0.y as f32, p.x as f32, p.y as f32)
                }
                PathEl::CurveTo(c0, c1, p) => pb.cubic_to(
                    c0.x as f32,
                    c0.y as f32,
                    c1.x as f32,
                    c1.y as f32,
                    p.x as f32,
                    p.y as f32,
                ),
                PathEl::ClosePath => pb.close(),
            }
        }
        pb.finish().unwrap()
    };
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    pixmap.encode_png()
}
