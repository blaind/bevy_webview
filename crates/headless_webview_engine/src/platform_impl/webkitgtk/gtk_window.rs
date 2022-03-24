use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use gdk::Device;
use gtk::{
    prelude::{ContainerExt, WidgetExt},
    traits::GtkWindowExt,
    Inhibit, Orientation,
};

use headless_webview::types::WindowSize;
use headless_webview::window::WindowId;
use headless_webview::HeadlessWindow;

use headless_webview::{window::WindowAttributes, Result};

use super::GtkWebview;

pub struct GtkWindow<T> {
    window_id: WindowId,
    pub(crate) inner: T,
    pub(crate) device: Device,
    pub(crate) has_events: Rc<AtomicBool>,
}

// see https://gist.github.com/mertyildiran/e83fc3091b355280ada63534432adcea
impl<T: ContainerExt + WidgetExt + GtkWindowExt> HeadlessWindow for GtkWindow<T> {
    type NativeWindow = T;
    type Webview = GtkWebview<T>;

    fn new(native_window: Self::NativeWindow, attributes: WindowAttributes) -> Result<Self>
    where
        Self: Sized,
    {
        let inner_size = attributes.get_inner_size();
        native_window.set_size_request(inner_size.width as i32, inner_size.height as i32);

        let has_events = Rc::new(AtomicBool::new(false));
        let window_has_events = has_events.clone();

        native_window.connect_draw(move |_, _| {
            //println!("{:?}: draw!", std::time::SystemTime::now(),);
            window_has_events.store(true, Ordering::SeqCst);
            Inhibit(false)
        });

        if attributes.transparent {
            if let Some(visual) = native_window.screen().and_then(|v| v.rgba_visual()) {
                native_window.set_visual(Some(&visual));
            } else {
                println!(
                    "WARN: could not find visual, even though transparent window was requested"
                );
            }
        }

        native_window.set_app_paintable(true);

        let window_box = gtk::Box::new(Orientation::Vertical, 0);
        native_window.add(&window_box);

        let display = gdk::Display::default().unwrap();
        let device_manager = display.default_seat().unwrap();
        let device = device_manager.pointer().unwrap();

        Ok(Self {
            window_id: WindowId(0),
            inner: native_window,
            has_events,
            device,
        })
    }

    fn id(&self) -> WindowId {
        self.window_id
    }

    fn inner_size(&self) -> WindowSize {
        let gdk_window = self.inner.window().unwrap();
        WindowSize::new(gdk_window.width() as u32, gdk_window.height() as u32)
    }

    fn resize(&self, new_size: WindowSize) -> Result<()> {
        self.inner
            .set_size_request(new_size.width as i32, new_size.height as i32);

        self.inner
            .resize(new_size.width as i32, new_size.height as i32);

        self.inner
            .window()
            .unwrap()
            .resize(new_size.width as i32, new_size.height as i32);

        Ok(())
    }
}
