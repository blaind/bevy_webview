use crate::types::WindowSize;
use crate::webview::EngineWebview;
use crate::Result;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u32);

/// Represents an underlying (headless) window
pub trait HeadlessWindow {
    type NativeWindow;
    type Webview: EngineWebview<Window = Self>;

    // Create a new window
    fn new(native_window: Self::NativeWindow, attributes: WindowAttributes) -> Result<Self>
    where
        Self: Sized;

    // Get window ID
    fn id(&self) -> WindowId;

    // Return the window size
    fn inner_size(&self) -> WindowSize;

    // Return window width
    fn width(&self) -> u32 {
        self.inner_size().width
    }

    // Return window height
    fn height(&self) -> u32 {
        self.inner_size().height
    }

    // Resize the window
    fn resize(&self, new_size: WindowSize) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct WindowAttributes {
    pub inner_size: Option<WindowSize>,
    pub transparent: bool,
}

impl WindowAttributes {
    pub fn get_inner_size(&self) -> WindowSize {
        self.inner_size
            .as_ref()
            .map(|v| v.clone())
            .unwrap_or(WindowSize::default())
    }
}

pub struct WindowBuilder<T: HeadlessWindow> {
    attributes: WindowAttributes,
    inner: T::NativeWindow,
    before_build_fn:
        Option<fn(T::NativeWindow, WindowAttributes) -> (T::NativeWindow, WindowAttributes)>,
}

impl<T: HeadlessWindow> WindowBuilder<T> {
    pub fn new(window: T::NativeWindow) -> Self {
        Self {
            attributes: Default::default(),
            inner: window,
            before_build_fn: None,
        }
    }

    pub fn add_before_build_fn(
        mut self,
        before_build_fn: fn(
            T::NativeWindow,
            WindowAttributes,
        ) -> (T::NativeWindow, WindowAttributes),
    ) -> Self {
        self.before_build_fn = Some(before_build_fn);
        self
    }

    pub fn with_inner_size(mut self, inner_size: WindowSize) -> Self {
        self.attributes.inner_size = Some(inner_size);
        self
    }

    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.attributes.transparent = transparent;
        self
    }

    pub fn build(self) -> Result<T> {
        if let Some(bfn) = self.before_build_fn {
            let (inner, attributes) = bfn(self.inner, self.attributes);
            Ok(T::new(inner, attributes)?)
        } else {
            Ok(T::new(self.inner, self.attributes)?)
        }
    }
}
