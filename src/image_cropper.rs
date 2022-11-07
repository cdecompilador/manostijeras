use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::*;
use winit::dpi::{PhysicalSize, PhysicalPosition, LogicalSize}; 
use winit::platform::run_return::EventLoopExtRunReturn;
use pixels::{Pixels, wgpu, PixelsBuilder, SurfaceTexture};

use crate::renderer::MasterRenderer;
use crate::image::Image;

/// Entry of the image cropper
pub struct ImageCropper {
    /// Window that we created
    pub window: winit::window::Window,

    /// The event loop
    pub event_loop: Option<EventLoop<()>>,

    /// Logical width of the inner part of the window
    pub width: u32,

    /// Logical height of the inner part of the window
    pub height: u32,

    /// Scale ratio
    pub ratio: f32,

    /// Pixels buffer
    pub pixbuf: pixels::Pixels,

    /// The loaded image to edit
    pub image: Image,

    /// Resized image to render
    pub render_image: Image,

    /// The cropped colored images
    image_crops: Vec<Image>,

    /// The container and manager of all the renderers
    renderer: Option<MasterRenderer>,
}

impl ImageCropper {
    pub fn new(
        image: Image
    ) -> Self {
        let event_loop = EventLoop::new();

        // Extract main monitor size and image dimensions
        let PhysicalSize {
            width: monitor_width,
            height: monitor_height
        } = event_loop.primary_monitor()
            .map(|monitor| monitor.size())
            .unwrap();
        let (mut window_width, mut window_height) = image.dimensions();

        // Calculate the ratio
        let mut ratio = 1.0;
        while window_width > monitor_width || window_height > monitor_height {
            ratio *= 0.5;
            window_width = (window_width as f32 * 0.5) as u32;
            window_height = (window_height as f32 * 0.5) as u32;
        }

        // Create the event loop and window
        let window = WindowBuilder::new()
            .with_resizable(false)
            .with_inner_size(
                LogicalSize::new(window_width, window_height))
            .with_title("Image Cropper")
            .build(&event_loop)
            .unwrap();

        // Get the inner physical size of the window and store it
        let PhysicalSize { 
            width,
            height
        } = window.inner_size();


        let mut render_image = image.clone();
        render_image.resize(ratio);

        // Create the pixels buffer
        let mut pixbuf = {
            let surface_texture = SurfaceTexture::new(width, height, &window);
            PixelsBuilder::new(width, height, surface_texture)
                .build().unwrap()
        };

        Self {
            window,
            event_loop: Some(event_loop),
            width,
            height,
            ratio,
            image,
            render_image,
            pixbuf,
            image_crops: Vec::new(),
            renderer: None
        }
    }

    pub fn handle_event(
        &mut self,
        event: Event<'_, ()>,
        control_flow: &mut ControlFlow
    ) {
        *control_flow = ControlFlow::Wait;

        // Get the handler
        let mut renderer = self.renderer.take().unwrap();

        // Handle events
        match event {
            Event::RedrawRequested(_) => {
                let PhysicalSize {
                    width,
                    height
                } = self.window.inner_size();
                self.pixbuf.resize_surface(width, height);
                self.pixbuf.resize_buffer(width, height);
                dbg!(self.window.inner_size());
                dbg!(self.render_image.dimensions());
                self.pixbuf.get_frame_mut()
                    .copy_from_slice(self.render_image.as_bytes());

                renderer.buff_render(self);

                self.pixbuf.render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);

                    renderer.gpu_render(
                        [self.width, self.height],
                        encoder,
                        render_target,
                        context
                    );

                    Ok(())
                }).unwrap();
            }
            Event::WindowEvent {
                ref event,
                ..
            } => {
                if renderer.color_picker.handle_event(event) {
                    renderer.request_redraw(self);
                    self.renderer = Some(renderer);
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        self.image_crops = 
                            renderer.regions.get_image_crops(self.ratio,
                                                             &self.image);
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved {
                        position: PhysicalPosition { x, y },
                        ..
                    } => {
                        renderer.mouse_move(self, *x as f32, *y as f32);
                    }
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    } => match button {
                        MouseButton::Left => {
                            renderer.mouse_left_click(self);
                        }
                        MouseButton::Right => {
                            renderer.mouse_right_click(self);
                        }
                        _ => {}
                    }
                    _ => {}
                };
            }
            _ => {}
        }

        self.renderer = Some(renderer);
    }

    pub fn run(mut self) -> anyhow::Result<Vec<Image>> {
        // Register the event handler
        self.renderer = Some(MasterRenderer::create(&mut self));

        // Handle events forever unless we get an error or the application
        // should exit
        loop {
            if let Some(mut event_loop) = self.event_loop.take() {
                if event_loop.run_return(|event, _, control_flow| {
                    self.handle_event(event, control_flow);
                }) != 0 {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(self.image_crops)
    }
}

