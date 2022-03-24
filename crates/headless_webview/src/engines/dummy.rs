use std::{
    rc::Rc,
    sync::{Mutex, RwLock},
};

use crate::{
    types::{KeyboardInput, MouseEvent, Texture, TextureFormat, WindowSize},
    webview::{
        web_context::{WebContext, WebContextData, WebContextImpl},
        EngineWebview, WebViewAttributes,
    },
    window::{HeadlessWindow, WindowAttributes, WindowBuilder, WindowId},
    Result,
};

pub fn dummy() -> WindowBuilder<DummyWindow> {
    WindowBuilder::new(())
}

pub struct DummyWindow {
    inner_size: RwLock<WindowSize>,
}

impl HeadlessWindow for DummyWindow {
    type NativeWindow = ();
    type Webview = DummyWebView;

    fn new(_native_window: Self::NativeWindow, attributes: WindowAttributes) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(DummyWindow {
            inner_size: RwLock::new(attributes.get_inner_size()),
        })
    }

    fn inner_size(&self) -> WindowSize {
        let inner_size = self.inner_size.read().unwrap();
        inner_size.clone()
    }

    fn resize(&self, new_size: WindowSize) -> Result<()> {
        let mut inner_size = self.inner_size.write().unwrap();
        *inner_size = new_size;

        Ok(())
    }

    fn id(&self) -> WindowId {
        WindowId(0)
    }
}

pub struct DummyWebView {
    window: Rc<DummyWindow>,
}

impl EngineWebview for DummyWebView {
    type Window = DummyWindow;
    type WebContext = DummyWebContext;

    fn new(
        window: Rc<Self::Window>,
        _webview: WebViewAttributes<Self::Window>,
        _web_context: Option<Rc<Mutex<WebContext<Self::WebContext>>>>,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(DummyWebView { window })
    }

    fn send_keyboard_input(&self, keyboard_input: KeyboardInput) {
        println!("Received keyboard event: {:?}", keyboard_input);
    }

    fn send_mouse_event(&self, mouse_event: MouseEvent) {
        println!("Received mouse event: {:?}", mouse_event);
    }

    fn window(&self) -> &Self::Window {
        &self.window
    }

    fn evaluate_script(&self, _js: &str) -> Result<()> {
        Ok(())
    }

    fn get_texture(&mut self) -> Result<Option<Texture>> {
        Ok(Some(Texture {
            width: self.window.width(),
            height: self.window.height(),
            format: TextureFormat::Rgba8,
            data: [50, 180, 50, 255].repeat((self.window.width() * self.window.height()) as usize),
        }))
    }

    fn tick_once(&mut self) {
        // nothing
    }

    fn version(&self) -> Result<String> {
        Ok(String::from("dummy-v0.0.1"))
    }

    fn resize(&self, new_size: WindowSize) -> Result<()> {
        self.window.resize(new_size)
    }

    fn close(&mut self) {
        // empty
    }

    fn load_html(&self, _html: String) {}
    fn load_uri(&self, _uri: String) {}

    fn reload(&self) {}

    fn send_mouse_position(&self, _position: crate::types::Vec2) {}

    fn set_is_visible(&mut self, _is_visible: bool) {}
}

pub struct DummyWebContext {}

impl WebContextImpl for DummyWebContext {
    fn new(_data: &WebContextData) -> Self {
        Self {}
    }

    fn set_allows_automation(&mut self, _flag: bool) {}
}
