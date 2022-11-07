use std::mem;

use tiny_skia::*;

use crate::image::Image;

#[derive(Debug)]
enum Line {
    Vertical { 
        x: f32,
        start_y: f32,
        end_y: f32 
    },
    Horizontal {
        y: f32,
        start_x: f32,
        end_x: f32
    }
}

#[derive(Debug)]
struct BoundLine {
    line: Line,
    margin: f32,
}

impl BoundLine {
    fn horizontal(y: f32, start_x: f32, end_x: f32, margin: f32) -> Self {
        Self {
            line: Line::Horizontal { 
                y,
                start_x,
                end_x 
            },
            margin
        }
    }

    fn vertical(x: f32, start_y: f32, end_y: f32, margin: f32) -> Self {
        Self {
            line: Line::Vertical {
                x,
                start_y,
                end_y
            },
            margin
        }
    }

    fn collides(&self, px: f32, py: f32) -> bool {
        match self.line {
            Line::Vertical {
                x,
                start_y,
                end_y
            } => {
                if px <= x + self.margin && px >= x - self.margin {
                    if py >= start_y - self.margin 
                            && py <= end_y + self.margin {
                        return true;
                    }
                }

                return false;
            }
            Line::Horizontal {
                y,
                start_x,
                end_x,
            } => {
                if py <= y + self.margin && py >= y - self.margin {
                    if px >= start_x - self.margin
                            && px <= end_x + self.margin {
                        return true;
                    }
                }

                return false;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegionState {
    Start { 
        x1: f32,
        y1: f32 
    },
    Complete { 
        x1: f32, 
        y1: f32, 
        x2: f32,
        y2: f32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub state: RegionState,
    pub color: egui::Color32
}

impl Region {
    fn start(x1: f32, y1: f32) -> Self {
        Self { 
            state: RegionState::Start { 
                x1,
                y1
            },
            color: egui::Color32::GRAY
        }
    }

    fn finish(&mut self, mut x2: f32, mut y2: f32) {
        match self.state {
            RegionState::Start { mut x1, mut y1 } => {
                if x1 > x2 {
                    mem::swap(&mut x1, &mut x2);
                }
                if y1 > y2 {
                    mem::swap(&mut y1, &mut y2);
                }
                self.state = RegionState::Complete { x1, y1, x2, y2 };
            }
            _ => panic!("This `Region` is already completed")
        }
    }

    fn update_color(&mut self, color: egui::Color32) {
        self.color = color;
    }

    fn path(&self) -> Option<Path> {
        match self.state {
            RegionState::Complete { x1, y1, x2, y2 } => {
                Some(PathBuilder::from_rect(
                        Rect::from_ltrb(x1, y1, x2, y2).unwrap()))
            }
            _ => None
        }
    }

    fn collides(&self, px: f32, py: f32) -> Option<bool> {
        for bline in self.blines(4.0)? {
            if bline.collides(px, py) {
                return Some(true);
            }
        }

        return Some(false);
    }

    fn blines(&self, margin: f32) -> Option<[BoundLine; 4]> {
        if let RegionState::Complete {
            x1,
            y1,
            x2,
            y2
        } = self.state {
            Some([
                BoundLine::horizontal(y1, x1, x2, margin),
                BoundLine::horizontal(y2, x1, x2, margin),
                BoundLine::vertical(x1, y1, y2, margin),
                BoundLine::vertical(x2, y1, y2, margin)
            ])
        } else {
            None
        }
    }
}

pub struct Regions {
    regions: Vec<Region>,
    selected_region: Option<usize>
}

impl Regions {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            selected_region: None
        }
    }

    pub fn start(&mut self, x1: f32, y1: f32) {
        self.regions.push(Region::start(x1, y1));
    }

    pub fn finish(&mut self, x2: f32, y2: f32) {
        if self.regions.is_empty() {
            panic!("Can't finish regions because there is no region in regions");
        }

        self.regions.last_mut().unwrap().finish(x2, y2);
    }

    pub fn is_finished(&self) -> bool {
        if self.regions.is_empty() {
            true
        } else {
            if let Some(
                RegionState::Complete { .. }
            ) = self.regions.last().map(|r| r.state) {
                true
            } else {
                false
            }
        }
    }

    /// Try to select the first region found that is collided by the mouse.
    ///
    /// Returns if it found any.
    pub fn select_collided_region(
        &mut self,
        px: f32, py: f32
    ) -> bool {
        for (idx, region) in self.regions.iter().enumerate() {
            dbg!(region, &region.collides(px, py));
            if let Some(true) = region.collides(px, py) {
                self.selected_region = Some(idx);
                return true;
            }
        }

        return false;
    }

    pub fn update_selected_color(&mut self, color: egui::Color32) {
        if let Some(idx) = self.selected_region {
            self.regions[idx].update_color(color);
        }
    }

    pub fn deselect(&mut self) {
        assert!(self.selected_region.is_some());
        self.selected_region = None;
    }

    pub fn render(&self, pixmap: &mut PixmapMut) {
        // Create paint color and stroke
        let mut paint = Paint::default();
        paint.set_color_rgba8(50, 127, 150, 255);
        let mut stroke = Stroke::default();
        stroke.width = 4.0;

        // Draw every region rect
        for region in &self.regions {
            let path = region.path();
            if path.is_none() {
                break;
            }
            let path = path.unwrap();

            // Use the color of the region
            paint.set_color_rgba8(
                region.color.r(),
                region.color.g(),
                region.color.b(),
                region.color.a()
            );

            pixmap.stroke_path(
                &path,
                &paint,
                &stroke,
                Transform::identity(),
                None
            );
        }
    }

    pub fn get_image_crops(&self, ratio: f32, original_image: &Image) -> Vec<Image> {
        let mut res = Vec::new();
        let mut c = 0;
        for region in &self.regions {
            res.push(original_image.extract_region(c, ratio, *region));
            c += 1;
        }

        res
    }
}

