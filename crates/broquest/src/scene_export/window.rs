//! A platform window that forwards everything to the real one and writes the
//! frame it is handed when an export was requested.

use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use futures::channel::oneshot;
use gpui::{
    A11yCallbacks, Bounds, Capslock, ComposedScene, Decorations, DispatchEventResult,
    ExternalDragPayload, GpuSpecs, Modifiers, Pixels, PlatformAtlas, PlatformCompositionSurface,
    PlatformDisplay, PlatformInput, PlatformInputHandler, PlatformSurfaceAttachment,
    PlatformWindow, Point, PromptButton, PromptLevel, RequestFrameOptions, ResizeEdge, Scene, Size,
    SystemWindowTab, TextInputConfiguration, TextInputStateChange, WindowAppearance,
    WindowBackgroundAppearance, WindowBounds, WindowControlArea, WindowControls, WindowDecorations,
    WindowInsets, WindowVisibility,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};

use super::RECORDER;
use super::text::ExportAtlas;

pub(super) struct ExportWindow {
    inner: Box<dyn PlatformWindow>,
    atlas: Arc<ExportAtlas>,
}

impl ExportWindow {
    pub(super) fn new(inner: Box<dyn PlatformWindow>) -> Self {
        // `PlatformWindow::sprite_atlas` hands out an `Arc`, though the atlas
        // only lives on the main thread.
        #[allow(clippy::arc_with_non_send_sync)]
        let atlas = Arc::new(ExportAtlas::new(inner.sprite_atlas()));
        Self { inner, atlas }
    }

    /// Writes `scene` if an export is waiting for a frame.
    fn export(&self, scene: &Scene) {
        let mut recorder = RECORDER.lock().unwrap();
        let Some((path, sender)) = recorder.request.take() else {
            return;
        };
        let viewport = self.inner.content_size();
        let result =
            super::dump::write(&recorder, scene, &path, viewport, self.inner.scale_factor());
        if let Err(error) = &result {
            tracing::error!(
                "Failed to export the frame to {}: {error:#}",
                path.display()
            );
        }
        sender.send(result).ok();
    }
}

impl HasWindowHandle for ExportWindow {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        self.inner.window_handle()
    }
}

impl HasDisplayHandle for ExportWindow {
    fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
        self.inner.display_handle()
    }
}

impl PlatformWindow for ExportWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        self.inner.bounds()
    }

    fn is_maximized(&self) -> bool {
        self.inner.is_maximized()
    }

    fn window_bounds(&self) -> WindowBounds {
        self.inner.window_bounds()
    }

    fn content_size(&self) -> Size<Pixels> {
        self.inner.content_size()
    }

    fn visual_viewport_bounds(&self) -> Bounds<Pixels> {
        self.inner.visual_viewport_bounds()
    }

    fn on_visual_viewport_changed(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_visual_viewport_changed(callback)
    }

    fn prepare_frame(&self) -> bool {
        self.inner.prepare_frame()
    }

    fn resize(&mut self, size: Size<Pixels>) {
        self.inner.resize(size)
    }

    fn scale_factor(&self) -> f32 {
        self.inner.scale_factor()
    }

    fn appearance(&self) -> WindowAppearance {
        self.inner.appearance()
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        self.inner.display()
    }

    fn mouse_position(&self) -> Point<Pixels> {
        self.inner.mouse_position()
    }

    fn modifiers(&self) -> Modifiers {
        self.inner.modifiers()
    }

    fn capslock(&self) -> Capslock {
        self.inner.capslock()
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        self.inner.set_input_handler(input_handler)
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.inner.take_input_handler()
    }

    fn set_text_input_configuration(&mut self, configuration: TextInputConfiguration) {
        self.inner.set_text_input_configuration(configuration)
    }

    fn prompt(
        &self,
        level: PromptLevel,
        msg: &str,
        detail: Option<&str>,
        answers: &[PromptButton],
    ) -> Option<oneshot::Receiver<usize>> {
        self.inner.prompt(level, msg, detail, answers)
    }

    fn activate(&self) {
        self.inner.activate()
    }

    fn request_attention(&self) {
        self.inner.request_attention()
    }

    fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    fn visibility(&self) -> WindowVisibility {
        self.inner.visibility()
    }

    fn is_hovered(&self) -> bool {
        self.inner.is_hovered()
    }

    fn background_appearance(&self) -> WindowBackgroundAppearance {
        self.inner.background_appearance()
    }

    fn set_title(&mut self, title: &str) {
        self.inner.set_title(title)
    }

    fn set_background_appearance(&self, background_appearance: WindowBackgroundAppearance) {
        self.inner.set_background_appearance(background_appearance)
    }

    fn minimize(&self) {
        self.inner.minimize()
    }

    fn zoom(&self) {
        self.inner.zoom()
    }

    fn toggle_fullscreen(&self) {
        self.inner.toggle_fullscreen()
    }

    fn is_fullscreen(&self) -> bool {
        self.inner.is_fullscreen()
    }

    fn frame_waker(&self) -> Option<Rc<dyn Fn()>> {
        self.inner.frame_waker()
    }

    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        self.inner.on_request_frame(callback)
    }

    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> DispatchEventResult>) {
        self.inner.on_input(callback)
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.inner.on_active_status_change(callback)
    }

    fn on_visibility_change(&self, callback: Box<dyn FnMut(WindowVisibility)>) {
        self.inner.on_visibility_change(callback)
    }

    fn on_hover_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.inner.on_hover_status_change(callback)
    }

    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        self.inner.on_resize(callback)
    }

    fn on_moved(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_moved(callback)
    }

    fn on_should_close(&self, callback: Box<dyn FnMut() -> bool>) {
        self.inner.on_should_close(callback)
    }

    fn on_hit_test_window_control(&self, callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
        self.inner.on_hit_test_window_control(callback)
    }

    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        self.inner.on_close(callback)
    }

    fn on_appearance_changed(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_appearance_changed(callback)
    }

    fn on_button_layout_changed(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_button_layout_changed(callback)
    }

    fn draw(&self, scene: &Scene) {
        self.export(scene);
        self.inner.draw(scene)
    }

    fn draw_composed(&self, scene: ComposedScene<'_>) {
        self.export(scene.scene());
        self.inner.draw_composed(scene)
    }

    fn enable_window_composition(&self) -> Result<()> {
        self.inner.enable_window_composition()
    }

    fn create_native_surface(&self) -> Result<Rc<dyn PlatformSurfaceAttachment>> {
        self.inner.create_native_surface()
    }

    fn set_composition_order(&self, fast_surfaces: &[PlatformCompositionSurface]) -> Result<()> {
        self.inner.set_composition_order(fast_surfaces)
    }

    fn schedule_frame(&self) {
        self.inner.schedule_frame()
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.atlas.clone()
    }

    fn is_subpixel_rendering_supported(&self) -> bool {
        self.inner.is_subpixel_rendering_supported()
    }

    fn get_title(&self) -> String {
        self.inner.get_title()
    }

    fn tabbed_windows(&self) -> Option<Vec<SystemWindowTab>> {
        self.inner.tabbed_windows()
    }

    fn tab_bar_visible(&self) -> bool {
        self.inner.tab_bar_visible()
    }

    fn set_edited(&mut self, edited: bool) {
        self.inner.set_edited(edited)
    }

    fn set_document_path(&self, path: Option<&std::path::Path>) {
        self.inner.set_document_path(path)
    }

    fn toggle_simple_fullscreen(&self) {
        self.inner.toggle_simple_fullscreen()
    }

    fn is_simple_fullscreen(&self) -> bool {
        self.inner.is_simple_fullscreen()
    }

    #[cfg(target_os = "macos")]
    fn set_traffic_light_position(&self, position: Point<Pixels>) {
        self.inner.set_traffic_light_position(position)
    }

    fn show_character_palette(&self) {
        self.inner.show_character_palette()
    }

    fn titlebar_double_click(&self, is_resizable: bool, is_minimizable: bool) {
        self.inner
            .titlebar_double_click(is_resizable, is_minimizable)
    }

    fn on_move_tab_to_new_window(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_move_tab_to_new_window(callback)
    }

    fn on_merge_all_windows(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_merge_all_windows(callback)
    }

    fn on_select_previous_tab(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_select_previous_tab(callback)
    }

    fn on_select_next_tab(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_select_next_tab(callback)
    }

    fn on_toggle_tab_bar(&self, callback: Box<dyn FnMut()>) {
        self.inner.on_toggle_tab_bar(callback)
    }

    fn merge_all_windows(&self) {
        self.inner.merge_all_windows()
    }

    fn move_tab_to_new_window(&self) {
        self.inner.move_tab_to_new_window()
    }

    fn toggle_window_tab_overview(&self) {
        self.inner.toggle_window_tab_overview()
    }

    fn set_tabbing_identifier(&self, identifier: Option<String>) {
        self.inner.set_tabbing_identifier(identifier)
    }

    fn native_window_state(&self) -> Option<Vec<u8>> {
        self.inner.native_window_state()
    }

    fn restore_native_window_state(&self, state: &[u8]) {
        self.inner.restore_native_window_state(state)
    }

    fn inner_window_bounds(&self) -> WindowBounds {
        self.inner.inner_window_bounds()
    }

    fn request_decorations(&self, decorations: WindowDecorations) {
        self.inner.request_decorations(decorations)
    }

    fn show_window_menu(&self, position: Point<Pixels>) {
        self.inner.show_window_menu(position)
    }

    fn start_window_move(&self) {
        self.inner.start_window_move()
    }

    fn can_start_external_drag(&self) -> bool {
        self.inner.can_start_external_drag()
    }

    fn start_external_drag(&self, payload: &ExternalDragPayload) -> bool {
        self.inner.start_external_drag(payload)
    }

    fn start_window_resize(&self, edge: ResizeEdge) {
        self.inner.start_window_resize(edge)
    }

    fn set_exclusive_zone(&self, zone: Pixels) {
        self.inner.set_exclusive_zone(zone)
    }

    fn set_input_region(&self, region: Option<&[Bounds<Pixels>]>) {
        self.inner.set_input_region(region)
    }

    fn window_decorations(&self) -> Decorations {
        self.inner.window_decorations()
    }

    fn set_app_id(&mut self, app_id: &str) {
        self.inner.set_app_id(app_id)
    }

    fn map_window(&mut self) -> Result<()> {
        self.inner.map_window()
    }

    fn window_controls(&self) -> WindowControls {
        self.inner.window_controls()
    }

    fn set_client_inset(&self, inset: Pixels) {
        self.inner.set_client_inset(inset)
    }

    fn gpu_specs(&self) -> Option<GpuSpecs> {
        self.inner.gpu_specs()
    }

    fn update_ime_position(&self, bounds: Bounds<Pixels>) {
        self.inner.update_ime_position(bounds)
    }

    fn insets(&self) -> WindowInsets {
        self.inner.insets()
    }

    fn on_insets_changed(&self, callback: Box<dyn FnMut(WindowInsets)>) {
        self.inner.on_insets_changed(callback)
    }

    fn set_back_handler(&self, callback: Box<dyn FnMut()>) {
        self.inner.set_back_handler(callback)
    }

    fn set_back_enabled(&self, enabled: bool) {
        self.inner.set_back_enabled(enabled)
    }

    fn show_soft_keyboard(&self) {
        self.inner.show_soft_keyboard()
    }

    fn hide_soft_keyboard(&self) {
        self.inner.hide_soft_keyboard()
    }

    fn text_input_state_changed(&self, change: TextInputStateChange) {
        self.inner.text_input_state_changed(change)
    }

    fn play_system_bell(&self) {
        self.inner.play_system_bell()
    }

    fn a11y_init(&self, callbacks: A11yCallbacks) {
        self.inner.a11y_init(callbacks)
    }

    fn a11y_tree_update(&self, tree_update: gpui::accesskit::TreeUpdate) {
        self.inner.a11y_tree_update(tree_update)
    }

    fn a11y_update_window_bounds(&self) {
        self.inner.a11y_update_window_bounds()
    }
}
