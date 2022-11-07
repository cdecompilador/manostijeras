use pixels::wgpu;
use tiny_skia::*;

use crate::color_picker::ColorPicker;
use crate::regions::Regions;
use crate::crosshair::Crosshair;
use crate::ImageCropper;

pub struct MasterRenderer {
    mouse_pos_x: f32,
    mouse_pos_y: f32,
    pub regions: Regions,
    pub color_picker: ColorPicker,
    pub crosshair: Crosshair,
}

impl MasterRenderer {
    /// Called when the window is created to create this handler
    pub fn create<'a>(app: &'a mut ImageCropper) -> Self {
        println!("Window created");

        Self {
            mouse_pos_x: 0.0,
            mouse_pos_y: 0.0,
            regions: Regions::new(),
            color_picker: ColorPicker::new(&app.event_loop.as_ref().unwrap()),
            crosshair: Crosshair::new(app.width as f32, app.height as f32),
        }
    }

    /// The mouse moved
    pub fn mouse_move(
        &mut self, 
        app: &mut ImageCropper,
        pos_x: f32, pos_y: f32
    ) {
        self.mouse_pos_x = pos_x;
        self.mouse_pos_y = pos_y;

        self.request_redraw(app);
    }

    pub fn mouse_left_click(
        &mut self,
        app: &mut ImageCropper 
    ) {
        if self.color_picker.show {
            self.regions.deselect();
            self.color_picker.show = false;
        } else {
            if self.regions.is_finished() {
                self.regions.start(
                    self.mouse_pos_x,
                    self.mouse_pos_y
                );
            } else {
                self.regions.finish(
                    self.mouse_pos_x,
                    self.mouse_pos_y
                );
            }
        }

        self.request_redraw(app);
    }

    pub fn mouse_right_click(
        &mut self,
        app: &mut ImageCropper 
    ) {
        if self.regions.select_collided_region(
            self.mouse_pos_x,
            self.mouse_pos_y
        ) {
            self.color_picker.show = true;
            self.request_redraw(app);
        }
    }

    pub fn buff_render(
        &mut self,
        app: &mut ImageCropper
    ) {
        let mut pixmap = PixmapMut::from_bytes(
            app.pixbuf.get_frame_mut(),
            app.width,
            app.height
        ).unwrap();

        self.regions.render(&mut pixmap);

        self.crosshair.render(
            &mut pixmap,
            self.mouse_pos_x,
            self.mouse_pos_y
        );
    }

    pub fn gpu_render(
        &mut self,
        size: [u32; 2],
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &pixels::PixelsContext
    ) {
        self.color_picker.render(
            size,
            encoder,
            render_target,
            context
        );
    }

    pub fn request_redraw(&mut self, app: &mut ImageCropper) {
        self.regions.update_selected_color(self.color_picker.color);
        self.color_picker.prepare(&app.window);
        app.window.request_redraw();
    }
}
