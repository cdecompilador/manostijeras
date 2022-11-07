use tiny_skia::*;

pub struct Crosshair {
    paths: (Path, Path),
}

impl Crosshair {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            paths: ({
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, -height);
                pb.line_to(0.0, height);
                pb.finish().unwrap()
            }, {
                let mut pb = PathBuilder::new();
                pb.move_to(-width, 0.0);
                pb.line_to(width, 0.0);
                pb.finish().unwrap()
            })
        }
    }

    pub fn render(&self, pixmap: &mut PixmapMut, x: f32, y: f32) {
        // Create paint color and stroke
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 10, 30, 255);
        let mut stroke = Stroke::default();
        stroke.width = 2.0;
        stroke.dash = StrokeDash::new(vec![1.0; 20], 1.0);

        pixmap.stroke_path(
            &self.paths.0,
            &paint,
            &stroke,
            Transform::from_translate(x, y),
            None
        );

        pixmap.stroke_path(
            &self.paths.1,
            &paint,
            &stroke,
            Transform::from_translate(x, y),
            None
        );
    }
}

