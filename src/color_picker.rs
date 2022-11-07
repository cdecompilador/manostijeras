use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::Window;
use egui::widgets::color_picker::{color_picker_color32, Alpha};
use pixels::PixelsContext;
use pixels::wgpu;

/// Manages all the state required to render egui over `Pixels`
pub struct ColorPicker {
    /// egui and egui-winit primitives
    egui_state: egui_winit::State,
    context: egui::Context,

    /// Paint jobs, the figures to render for wgpu
    clipped_primitives: Vec<egui::ClippedPrimitive>,

    /// Changes on textures
    textures_delta: egui::TexturesDelta,

    /// User data
    pub color: egui::Color32,

    /// Used to know if there is need to render ui elements
    pub show: bool
}

impl ColorPicker {
    /// Initialize egui
    pub fn new(
        event_loop: &EventLoop<()>
    ) -> Self {
        let context = egui::Context::default();
        let egui_state = egui_winit::State::new(event_loop);

        Self {
            egui_state,
            context,
            clipped_primitives: Vec::new(),
            textures_delta: egui::TexturesDelta::default(),
            color: egui::Color32::GRAY,
            show: false
        }
    }

    /// Handle a event and return if it is exclusive to egui or should be
    /// processed by underlying elements
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_state.on_event(&self.context, event)
    }

    /// Prepare the egui primitives for rendering, process all the received
    /// input till this funcion is called and update all the commands that
    /// will be sent at `self.render()`
    pub fn prepare(&mut self, window: &Window) {
        let mut color = self.color.clone();

        // Extract (and clear) the egui captured raw input
        let raw_input = self.egui_state.take_egui_input(&window);

        // Process that input and create all the paint jobs required to draw a
        // new frame, also the changes issued by us for exaple the color pick
        let output = self.context.run(raw_input, |egui_ctx| {
            color = self.ui(egui_ctx);
        });
        self.color = color;

        // Do any external output issued from winit like for example updating
        // the cursor, copy text to clipboard, open URL, etc ...
        self.egui_state.handle_platform_output(
            &window,
            &self.context,
            output.platform_output
        );

        // Create the new paint jobs
        self.clipped_primitives = 
            self.context.tessellate(output.shapes);

        // Append texture deltas, ...
        self.textures_delta.append(
            output.textures_delta
        );
    }

    /// Egui ui elements to render, produces no elements if there is no need
    /// to render
    ///
    /// Returns the user input
    fn ui(&self, ctx: &egui::Context) -> egui::Color32 {
        // Check if there really is need for UI
        if !self.show {
            return self.color.clone();
        }

        // Render the color picker, and return the picked color
        let mut color = self.color.clone();
        egui::Window::new("My window")
            // .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| 
        {
            color_picker_color32(
                ui,
                &mut color,
                Alpha::Opaque
            );
        });

        color
    }

    /// Render the egui elements on the render target
    pub fn render(
        &self,
        size: [u32; 2],
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        PixelsContext {
            device,
            queue,
            ..
        }: &PixelsContext
    ) {
        // Create egui renderer
        let mut rpass = egui_wgpu::renderer::RenderPass::new(
            device,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            1
        );

        // Create a descriptor of the screen, that will tell the renderer how
        // to render its elements (scale, position) on the texture
        let pixels_per_point = self.egui_state.pixels_per_point();
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point
        };

        // Update vertices, indices, uniforms, etc ...
        rpass.update_buffers(
            device,
            queue,
            &self.clipped_primitives,
            &screen_descriptor
        );

        // Update textures scheduled for updating
        for (id, image_delta) in &self.textures_delta.set {
            rpass.update_texture(
                device,
                queue,
                *id,
                image_delta
            );
        }

        // Send draw commands to the GPU
        rpass.execute(
            encoder,
            render_target,
            &self.clipped_primitives,
            &screen_descriptor,
            None
        );

        // Free textures scheduled for free
        for id in &self.textures_delta.free {
            rpass.free_texture(id);
        }
    }
}
