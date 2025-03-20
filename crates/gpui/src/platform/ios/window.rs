use crate::platform::ios::renderer;
use crate::{
    platform::PlatformInputHandler, AnyWindowHandle, Bounds, ForegroundExecutor, Modifiers, Pixels,
    PlatformAtlas, PlatformDisplay, PlatformInput, PlatformWindow, Point, PromptLevel,
    RequestFrameOptions, ScaledPixels, Size, WindowAppearance, WindowBackgroundAppearance,
    WindowBounds,
};
use futures::channel::oneshot;
use objc2::rc::Retained;
use objc2_ui_kit::{UITraitCollection, UIView, UIWindowScene};
use parking_lot::Mutex;
use raw_window_handle as rwh;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Arc;

struct IosWindowState {
    handle: AnyWindowHandle,
    executor: ForegroundExecutor,
    renderer: renderer::Renderer,
    request_frame_callback: Option<Box<dyn FnMut(RequestFrameOptions)>>,
    event_callback: Option<Box<dyn FnMut(PlatformInput) -> crate::DispatchEventResult>>,
    activate_callback: Option<Box<dyn FnMut(bool)>>,
    resize_callback: Option<Box<dyn FnMut(Size<Pixels>, f32)>>,
    moved_callback: Option<Box<dyn FnMut()>>,
    should_close_callback: Option<Box<dyn FnMut() -> bool>>,
    close_callback: Option<Box<dyn FnOnce()>>,
    appearance_changed_callback: Option<Box<dyn FnMut()>>,
    input_handler: Option<PlatformInputHandler>,
    scene: Retained<UIWindowScene>,
    view: Retained<UIView>,
}

pub(crate) struct IosWindow(Arc<Mutex<IosWindowState>>);

impl PlatformWindow for IosWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        todo!()
    }

    fn window_bounds(&self) -> WindowBounds {
        todo!()
    }

    fn content_size(&self) -> Size<Pixels> {
        todo!()
    }

    fn scale_factor(&self) -> f32 {
        todo!()
    }

    fn appearance(&self) -> WindowAppearance {
        let style = unsafe { UITraitCollection::currentTraitCollection().userInterfaceStyle() };
        WindowAppearance::from_native(style)
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        None
    }

    fn mouse_position(&self) -> Point<Pixels> {
        // We would probably use UIPointerInteraction here.
        Point::default()
    }

    fn modifiers(&self) -> Modifiers {
        // On iOS, getting modifiers doesn't work the same way as on macOS.
        // Do not implemented for now
        Modifiers::default()
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        self.0.lock().input_handler = Some(input_handler);
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.0.lock().input_handler.take()
    }

    fn prompt(
        &self,
        level: PromptLevel,
        msg: &str,
        detail: Option<&str>,
        answers: &[&str],
    ) -> Option<oneshot::Receiver<usize>> {
        None
    }

    fn activate(&self) {
        let window = unsafe { self.0.lock().scene.windows().firstObject() };
        self.0
            .lock()
            .executor
            .clone()
            .spawn(async move { window.map(|w| w.makeKeyAndVisible()) });
    }

    fn is_active(&self) -> bool {
        unsafe {
            self.0
                .lock()
                .scene
                .windows()
                .firstObject()
                .map_or(false, |w| w.isKeyWindow())
        }
    }

    fn is_hovered(&self) -> bool {
        false
    }

    fn set_title(&mut self, _: &str) {
        // It's possible to set subtitle on iPad task manager.
    }

    fn set_background_appearance(&self, _: WindowBackgroundAppearance) {}

    // I haven't figure out a way to manage window size on iPad programmatically.
    // And it's obvious not possible on iPhone.
    // So it doesn't matter what we return here as we cannot do anything with it.
    fn is_maximized(&self) -> bool {
        false
    }

    fn minimize(&self) {}

    fn zoom(&self) {}

    fn toggle_fullscreen(&self) {}

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        self.0.lock().request_frame_callback = Some(callback);
    }

    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> crate::DispatchEventResult>) {
        self.0.lock().event_callback = Some(callback);
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.lock().activate_callback = Some(callback);
    }

    fn on_hover_status_change(&self, _: Box<dyn FnMut(bool)>) {}

    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        self.0.lock().resize_callback = Some(callback);
    }

    fn on_moved(&self, callback: Box<dyn FnMut()>) {
        self.0.lock().moved_callback = Some(callback);
    }

    fn on_should_close(&self, callback: Box<dyn FnMut() -> bool>) {
        self.0.lock().should_close_callback = Some(callback);
    }

    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        self.0.lock().close_callback = Some(callback);
    }

    fn on_appearance_changed(&self, callback: Box<dyn FnMut()>) {
        self.0.lock().appearance_changed_callback = Some(callback);
    }

    fn draw(&self, scene: &crate::Scene) {
        self.0.lock().renderer.draw(scene);
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.0.lock().renderer.sprite_atlas().clone()
    }

    fn gpu_specs(&self) -> Option<crate::GpuSpecs> {
        None
    }

    fn update_ime_position(&self, _bounds: Bounds<ScaledPixels>) {}
}

impl rwh::HasWindowHandle for IosWindow {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        unsafe {
            Ok(rwh::WindowHandle::borrow_raw(rwh::RawWindowHandle::UiKit(
                rwh::UiKitWindowHandle::new(NonNull::new_unchecked(self.0.lock().view.as_ref()
                    as *const UIView
                    as *mut _)),
            )))
        }
    }
}

impl rwh::HasDisplayHandle for IosWindow {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        unsafe {
            Ok(rwh::DisplayHandle::borrow_raw(
                rwh::UiKitDisplayHandle::new().into(),
            ))
        }
    }
}
