use std::path::{PathBuf, Path};

use anyhow::*;
use image::{Rgba, RgbaImage};
use image::io::Reader as ImageReader;

use crate::regions::{RegionState, Region};

const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

fn is_color(
    match_color: Rgba<u8>, 
    in_color: Rgba<u8>, 
    margin: u8
) -> bool {
    for i in 0..3 {
        if in_color[i] <= match_color[i].saturating_add(margin) 
                && in_color[i] >= match_color[i].saturating_sub(margin) {
            return true;
        }
    }

    return false;
}

#[derive(Clone)]
pub struct Image {
    image_buffer: RgbaImage,
    path: PathBuf,
}

impl Image {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let image_buffer = ImageReader::open(path.as_ref())?
            .decode()?
            .into_rgba8();

        Ok(Self {
            image_buffer,
            path: path.as_ref().to_owned()
        })
    }

    pub fn resize(&mut self, ratio: f32) {
        let (mut new_width, mut new_height) = self.dimensions();
        new_width = (new_width as f32 * ratio) as u32;
        new_height = (new_height as f32 * ratio) as u32;
        let new_image_buffer = image::imageops::resize(
            &self.image_buffer,
            new_width, new_height,
            image::imageops::FilterType::Nearest
        );

        self.image_buffer = new_image_buffer;
    }

    pub fn extract_region(
        &self,
        counter: u32,
        ratio: f32,
        region: Region
    ) -> Self {
        let mut path = self.path.clone();
        let name = format!("{}-{}.png", 
            path.file_stem().unwrap().to_str().unwrap(),
            counter);
        path.set_file_name(name);

        let (start_row, start_col, width, height) = match region {
            Region {
                state: RegionState::Complete {
                    x1,
                    y1,
                    x2,
                    y2
                },
                color
            } => {
                (
                    (x1 as f32 / ratio) as u32,
                    (y1 as f32 / ratio) as u32,
                    ((x2 - x1) as f32 / ratio) as u32,
                    ((y2 - y1) as f32 / ratio) as u32
                )
            },
            _ => panic!("Unexpected incomplete region")
        };

        let mut new_image_buffer = image::imageops::crop_imm(
            &self.image_buffer, 
            start_row, start_col,
            width, height
        ).to_image();

        for color in new_image_buffer.pixels_mut() {
            if is_color(BLACK, *color, 200) {
                let (r, g, b, a) = region.color.to_tuple();
                *color = Rgba([r, g, b, a]);
            } else {
                *color = TRANSPARENT;
            }
        }

        Self {
            image_buffer: new_image_buffer,
            path
        }
    }

    pub fn save(&self, out_dir: &PathBuf) -> Result<()> {
        let final_path = out_dir.join(&self.path);
        dbg!(&final_path);
        self.image_buffer.save(final_path)?;

        Ok(())
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.image_buffer.dimensions()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.image_buffer.as_raw()
    }
}
