use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "windows")]
use egui_wgpu::wgpu::Dx12SwapchainKind;
use egui_wgpu::wgpu::PresentMode;
#[cfg(target_os = "windows")]
use egui_wgpu::wgpu::{Backends, Dx12SwapchainKind};
use egui_wgpu::winit::Painter;
use egui_wgpu::{RendererOptions, WgpuConfiguration, WgpuSetup};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{Window as WinitWindow, WindowButtons, WindowId, WindowLevel};

use super::Error;
use super::diagnostics;
use super::render_state::RenderState;

/// Builds the wgpu setup for an overlay window.
///
/// On Windows the default DirectX 12 swapchain is tied directly to the window and cannot be
/// transparent, so the overlay would draw as an opaque white box over everything. A
/// DirectComposition swapchain supports transparency.
fn wgpu_setup(event_loop: &ActiveEventLoop) -> WgpuSetup {
    #[allow(unused_mut)]
    let mut setup = WgpuSetup::from_display_handle(event_loop.owned_display_handle());

    #[cfg(target_os = "windows")]
    if let WgpuSetup::CreateNew(create_new) = &mut setup {
        // Vulkan surfaces on Windows are always opaque, so use DirectX 12 only.
        create_new.instance_descriptor.backends = Backends::DX12;
        create_new
            .instance_descriptor
            .backend_options
            .dx12
            .presentation_system = Dx12SwapchainKind::DxgiFromVisual;
    }

    setup
}

/// Checks, before the overlay is ever shown, that this display can draw a see-through window.
///
/// An overlay that can't be transparent would cover the screen with an opaque box, so when this
/// fails the overlay is simply not shown. Everything it learns goes to the diagnostics log.
#[cfg(target_os = "windows")]
async fn verify_transparent_surface(
    event_loop: &ActiveEventLoop,
    window: &Arc<WinitWindow>,
) -> Result<(), String> {
    use egui_wgpu::wgpu::{CompositeAlphaMode, RequestAdapterOptions};

    let instance = wgpu_setup(event_loop);
    let WgpuSetup::CreateNew(_) = &instance else {
        return Ok(());
    };
    let instance = instance.new_instance().await;
    let surface = instance
        .create_surface(window.clone())
        .map_err(|error| format!("couldn't create a surface: {error}"))?;
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .map_err(|error| format!("no usable graphics adapter: {error}"))?;
    let info = adapter.get_info();
    let modes = surface.get_capabilities(&adapter).alpha_modes;
    diagnostics::log(format!(
        "overlay adapter: {} ({:?}, {:?}), driver {} {}; alpha modes: {modes:?}",
        info.name, info.backend, info.device_type, info.driver, info.driver_info
    ));

    if modes.contains(&CompositeAlphaMode::PreMultiplied)
        || modes.contains(&CompositeAlphaMode::PostMultiplied)
    {
        Ok(())
    } else {
        Err("this graphics setup can't draw a transparent window".into())
    }
}

/// A transparent click-through overlay window for one monitor.
///
/// `Window` owns the native winit window plus egui/wgpu integration required to render into it. It
/// deliberately does not own highlighter drawing decisions; those are supplied by `RenderState`
/// during each redraw.
///
/// Each window owns its own egui context. A painter only receives the texture updates (such as
/// font atlas changes) from passes run on its own window, so windows that share one context end
/// up drawing with incomplete textures.
pub struct Window {
    inner: Arc<WinitWindow>,
    egui_state: egui_winit::State,
    painter: Painter,
    viewport_id: egui::ViewportId,
}

impl Window {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        monitor: MonitorHandle,
        visible: bool,
    ) -> Result<Self, Error> {
        let context = super::fonts::context();
        let position = monitor.position();
        let size = monitor.size();
        let attributes = WinitWindow::default_attributes()
            .with_title("Gramflow")
            .with_inner_size(size)
            .with_position(position)
            .with_resizable(false)
            .with_enabled_buttons(WindowButtons::empty())
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_active(false);
        // The overlay is not a real window; keep it out of the Windows taskbar. It starts hidden and
        // without a Windows-drawn background, so it can never appear as an opaque white box: it is
        // only shown once the display is verified to support transparency (see below).
        #[cfg(target_os = "windows")]
        let attributes = attributes
            .with_skip_taskbar(true)
            .with_no_redirection_bitmap(true)
            .with_visible(false);
        let window = Arc::new(event_loop.create_window(attributes)?);
        diagnostics::log(format!(
            "overlay window for monitor at {},{} size {}x{} scale {}",
            position.x,
            position.y,
            size.width,
            size.height,
            window.scale_factor()
        ));

        #[cfg(target_os = "windows")]
        if let Err(reason) = verify_transparent_surface(event_loop, &window).await {
            diagnostics::log(format!("overlay disabled: {reason}"));
            return Err(Error::Overlay(reason));
        }

        window.set_outer_position(PhysicalPosition::new(position.x, position.y));
        let _ = window.request_inner_size(PhysicalSize::new(size.width, size.height));
        window.set_cursor_hittest(false)?;
        let viewport_id = egui::ViewportId::from_hash_of(window.id());

        let egui_state = egui_winit::State::new(
            context.clone(),
            viewport_id,
            event_loop,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );

        let mut painter = Painter::new(
            context,
            WgpuConfiguration {
                present_mode: PresentMode::Fifo,
                wgpu_setup: wgpu_setup(event_loop),
                ..Default::default()
            },
            true,
            RendererOptions::default(),
        )
        .await;
        painter
            .set_window(viewport_id, Some(window.clone()))
            .await?;
        window.request_redraw();
        #[cfg(target_os = "windows")]
        window.set_visible(true);

        Ok(Self {
            inner: window,
            egui_state,
            painter,
            viewport_id,
        })
    }

    pub fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    /// Restore overlays without making them key or interrupting typing in the underlying app.
    #[cfg(target_os = "macos")]
    pub fn set_visible(&self, visible: bool) {
        use objc2_app_kit::NSView;
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let handle = self.inner.window_handle().expect("overlay window handle");
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            unreachable!("macOS overlay must have an AppKit window handle");
        };
        // SAFETY: Winit owns this live NSView, and overlay lifecycle operations run on the main
        // event-loop thread. Keep the window handle alive while accessing the view and its window.
        let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
        let window = view.window().expect("overlay view must belong to a window");
        if visible {
            window.orderFrontRegardless();
        } else {
            window.orderOut(None);
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn set_visible(&self, visible: bool) {
        self.inner.set_visible(visible);
    }

    /// Controls whether the transparent overlay can receive pointer events.
    ///
    /// Highlight windows are click-through by default so the underlying app remains usable. The
    /// window manager temporarily enables hit-testing when global cursor polling shows the pointer
    /// is over an interactive highlight or popup.
    pub fn set_cursor_hittest(&self, enabled: bool) -> Result<(), Error> {
        self.inner.set_cursor_hittest(enabled)?;

        Ok(())
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        let response = self.egui_state.on_window_event(&self.inner, event);

        // RedrawRequested is already being serviced. Egui marks it as repaint-worthy too;
        // forwarding that response would perpetually submit frames even with no UI changes.
        if response.repaint && !matches!(event, WindowEvent::RedrawRequested) {
            self.inner.request_redraw();
        }

        if let WindowEvent::Resized(size) = event
            && let (Some(width), Some(height)) =
                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        {
            self.painter
                .on_window_resized(self.viewport_id, width, height);
            self.inner.request_redraw();
        }
    }

    /// The window's bounds in the screen coordinates that lints and the cursor use.
    ///
    /// On platforms that cannot report where the window is, this assumes the window is at the
    /// screen's origin.
    pub fn screen_rect(&self) -> egui::Rect {
        let pixels_per_point =
            egui_winit::pixels_per_point(self.egui_state.egui_ctx(), &self.inner);

        egui_winit::inner_rect_in_points(&self.inner, pixels_per_point).unwrap_or_else(|| {
            let size = self.inner.inner_size();
            egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size.width as f32, size.height as f32) / pixels_per_point,
            )
        })
    }

    /// Draws `render_state` into this window, which is `screen_rects[index]`.
    ///
    /// Returns egui's next repaint delay for this viewport. `Duration::MAX` means there is no
    /// scheduled repaint; input and OS invalidation can still request a frame independently.
    pub fn render(
        &mut self,
        render_state: &mut RenderState,
        screen_rects: &[egui::Rect],
        index: usize,
    ) -> Duration {
        let context = self.egui_state.egui_ctx().clone();
        let input = self.egui_state.take_egui_input(&self.inner);
        let output = context.run_ui(input, |ui| {
            render_state.render(ui, screen_rects, index);
        });

        let repaint_delay = output
            .viewport_output
            .get(&self.viewport_id)
            .map_or(Duration::MAX, |viewport| viewport.repaint_delay);

        self.egui_state
            .handle_platform_output(&self.inner, output.platform_output);

        let clipped_primitives = context.tessellate(output.shapes, output.pixels_per_point);
        self.painter.paint_and_update_textures(
            self.viewport_id,
            output.pixels_per_point,
            [0.0, 0.0, 0.0, 0.0],
            &clipped_primitives,
            &output.textures_delta,
            Vec::new(),
        );

        repaint_delay
    }
}
