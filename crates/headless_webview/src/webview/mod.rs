use std::rc::Rc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

use serde_json::Value;
use url::Url;

// :set diffopt+=iwhite
use crate::http::{Request as HttpRequest, Response as HttpResponse};
use crate::types::{KeyboardInput, MouseEvent, Texture, TickMode, Vec2, WindowSize};
use crate::window::HeadlessWindow;
use crate::{Error, Result};

pub mod web_context;

use web_context::{WebContext, WebContextImpl};

pub struct WebViewAttributes<T: HeadlessWindow> {
    /// Whether the WebView should have a custom user-agent.
    pub user_agent: Option<String>,
    /// Background color
    pub color: Color,
    /// Whether load the provided URL to [`WebView`].
    pub url: Option<Url>,
    /// Whether load the provided html string to [`WebView`].
    /// This will be ignored if the `url` is provided.
    ///
    /// # Warning
    /// The loaded from html string will have different Origin on different platforms. And
    /// servers which enforce CORS will need to add exact same Origin header in `Access-Control-Allow-Origin`
    /// if you wish to send requests with native `fetch` and `XmlHttpRequest` APIs. Here are the
    /// different Origin headers across platforms:
    ///
    /// - macOS: `http://localhost`
    /// - Linux: `http://localhost`
    /// - Windows: `null`
    pub html: Option<String>,
    /// Initialize javascript code when loading new pages. When webview load a new page, this
    /// initialization code will be executed. It is guaranteed that code is executed before
    /// `window.onload`.
    pub initialization_scripts: Vec<String>,
    /// Register custom file loading protocols with pairs of scheme uri string and a handling
    /// closure.
    ///
    /// The closure takes a url string slice, and returns a two item tuple of a vector of
    /// bytes which is the content and a mimetype string of the content.
    ///
    /// # Warning
    /// Pages loaded from custom protocol will have different Origin on different platforms. And
    /// servers which enforce CORS will need to add exact same Origin header in `Access-Control-Allow-Origin`
    /// if you wish to send requests with native `fetch` and `XmlHttpRequest` APIs. Here are the
    /// different Origin headers across platforms:
    ///
    /// - macOS: `<scheme_name>://<path>` (so it will be `wry://examples` in `custom_protocol` example)
    /// - Linux: Though it's same as macOS, there's a [bug] that Origin header in the request will be
    /// empty. So the only way to pass the server is setting `Access-Control-Allow-Origin: *`.
    /// - Windows: `https://<scheme_name>.<path>` (so it will be `https://wry.examples` in `custom_protocol` example)
    ///
    /// [bug]: https://bugs.webkit.org/show_bug.cgi?id=229034
    pub custom_protocols: Vec<(String, Box<dyn Fn(&HttpRequest) -> Result<HttpResponse>>)>,
    /// Set the RPC handler to Communicate between the host Rust code and Javascript on webview.
    ///
    /// The communication is done via [JSON-RPC](https://www.jsonrpc.org). Users can use this to register an incoming
    /// request handler and reply with responses that are passed back to Javascript. On the Javascript
    /// side the client is exposed via `window.rpc` with two public methods:
    ///
    /// 1. The `call()` function accepts a method name and parameters and expects a reply.
    /// 2. The `notify()` function accepts a method name and parameters but does not expect a reply.
    ///
    /// Both functions return promises but `notify()` resolves immediately.
    pub rpc_handler: Option<Box<dyn Fn(&T, RpcRequest) -> Option<RpcResponse>>>,

    /// Enables clipboard access for the page rendered on **Linux** and **Windows**.
    ///
    /// macOS doesn't provide such method and is always enabled by default. But you still need to add menu
    /// item accelerators to use shortcuts.
    pub clipboard: bool,
}

impl<T: HeadlessWindow> Default for WebViewAttributes<T> {
    fn default() -> Self {
        Self {
            user_agent: None,
            color: Color::default(),
            url: None,
            html: None,
            initialization_scripts: vec![],
            custom_protocols: vec![],
            rpc_handler: None,
            clipboard: false,
        }
    }
}

/// Builder type of [`WebView`].
///
/// [`WebViewBuilder`] / [`WebView`] are the basic building blocks to constrcut WebView contents and
/// scripts for those who prefer to control fine grained window creation and event handling.
/// [`WebViewBuilder`] privides ability to setup initialization before web engine starts.
pub struct WebviewBuilder<T: HeadlessWindow> {
    pub webview: WebViewAttributes<T>,
    web_context: Option<Rc<Mutex<WebContext<<T::Webview as EngineWebview>::WebContext>>>>,
    window: T,
}

impl<T> WebviewBuilder<T>
where
    T: HeadlessWindow,
{
    pub fn new(window: T) -> Result<Self> {
        let webview = WebViewAttributes::default();
        let web_context = None;

        Ok(Self {
            webview,
            web_context,
            window,
        })
    }

    /// Sets the background color
    pub fn with_color(mut self, color: Color) -> Self {
        self.webview.color = color;
        self
    }

    /// Initialize javascript code when loading new pages. When webview load a new page, this
    /// initialization code will be executed. It is guaranteed that code is executed before
    /// `window.onload`.
    pub fn with_initialization_script(mut self, js: &str) -> Self {
        self.webview.initialization_scripts.push(js.to_string());
        self
    }

    /// Register custom file loading protocols with pairs of scheme uri string and a handling
    /// closure.
    ///
    /// The closure takes a url string slice, and returns a two item tuple of a
    /// vector of bytes which is the content and a mimetype string of the content.
    ///
    /// # Warning
    /// Pages loaded from custom protocol will have different Origin on different platforms. And
    /// servers which enforce CORS will need to add exact same Origin header in `Access-Control-Allow-Origin`
    /// if you wish to send requests with native `fetch` and `XmlHttpRequest` APIs. Here are the
    /// different Origin headers across platforms:
    ///
    /// - macOS: `<scheme_name>://<path>` (so it will be `wry://examples` in `custom_protocol` example)
    /// - Linux: Though it's same as macOS, there's a [bug] that Origin header in the request will be
    /// empty. So the only way to pass the server is setting `Access-Control-Allow-Origin: *`.
    /// - Windows: `https://<scheme_name>.<path>` (so it will be `https://wry.examples` in `custom_protocol` example)
    ///
    /// [bug]: https://bugs.webkit.org/show_bug.cgi?id=229034
    #[cfg(feature = "protocol")]
    pub fn with_custom_protocol<F>(mut self, name: String, handler: F) -> Self
    where
        F: Fn(&HttpRequest) -> Result<HttpResponse> + 'static,
    {
        self.webview
            .custom_protocols
            .push((name, Box::new(handler)));
        self
    }

    /// Set the RPC handler to Communicate between the host Rust code and Javascript on webview.
    ///
    /// The communication is done via [JSON-RPC](https://www.jsonrpc.org). Users can use this to register an incoming
    /// request handler and reply with responses that are passed back to Javascript. On the Javascript
    /// side the client is exposed via `window.rpc` with two public methods:
    ///
    /// 1. The `call()` function accepts a method name and parameters and expects a reply.
    /// 2. The `notify()` function accepts a method name and parameters but does not expect a reply.
    ///
    /// Both functions return promises but `notify()` resolves immediately.
    pub fn with_rpc_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&T, RpcRequest) -> Option<RpcResponse> + 'static,
    {
        self.webview.rpc_handler = Some(Box::new(handler));
        self
    }

    /// Load the provided URL when the builder calling [`WebViewBuilder::build`] to create the
    /// [`WebView`]. The provided URL must be valid.
    pub fn with_url(mut self, url: &str) -> Result<Self> {
        self.webview.url = Some(Url::parse(url)?);
        Ok(self)
    }

    /// Load the provided HTML string when the builder calling [`WebViewBuilder::build`] to create the
    /// [`WebView`]. This will be ignored if `url` is already provided.
    ///
    /// # Warning
    /// The Page loaded from html string will have different Origin on different platforms. And
    /// servers which enforce CORS will need to add exact same Origin header in `Access-Control-Allow-Origin`
    /// if you wish to send requests with native `fetch` and `XmlHttpRequest` APIs. Here are the
    /// different Origin headers across platforms:
    ///
    /// - macOS: `http://localhost`
    /// - Linux: `http://localhost`
    /// - Windows: `null`
    pub fn with_html(mut self, html: impl Into<String>) -> Result<Self> {
        self.webview.html = Some(html.into());
        Ok(self)
    }

    /// Set the web context that can share with multiple [`WebView`]s.
    pub fn with_web_context(
        mut self,
        web_context: Rc<Mutex<WebContext<<T::Webview as EngineWebview>::WebContext>>>,
    ) -> Self {
        self.web_context = Some(web_context);
        self
    }

    /// Set a custom [user-agent](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent) for the WebView.
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.webview.user_agent = Some(user_agent.to_string());
        self
    }

    /// Consume the builder and create the [`WebView`].
    ///
    /// Platform-specific behavior:
    ///
    /// - **Unix:** This method must be called in a gtk thread. Usually this means it should be
    /// called in the same thread with the [`EventLoop`] you create.
    ///
    /// [`EventLoop`]: crate::application::event_loop::EventLoop
    pub fn build(mut self) -> Result<T::Webview> {
        if self.webview.rpc_handler.is_some() {
            self.webview
                .initialization_scripts
                .push(include_str!("javascript/rpc.js").to_string());
        }
        Ok(T::Webview::new(
            Rc::new(self.window),
            self.webview,
            self.web_context,
        )?)
    }
}

pub trait EngineWebview {
    type Window: HeadlessWindow;
    type WebContext: WebContextImpl;

    fn new(
        window: Rc<Self::Window>,
        attributes: WebViewAttributes<Self::Window>,
        web_context: Option<Rc<Mutex<WebContext<Self::WebContext>>>>,
    ) -> Result<Self>
    where
        Self: Sized;

    /// Get the [`Window`] associate with the [`WebView`]. This can let you perform window related
    /// actions.
    fn window(&self) -> &Self::Window;

    /// Evaluate and run javascript code. Must be called on the same thread who created the
    /// [`WebView`]. Use [`EventLoopProxy`] and a custom event to send scripts from other threads.
    ///
    /// [`EventLoopProxy`]: crate::application::event_loop::EventLoopProxy
    fn evaluate_script(&self, js: &str) -> Result<()>;

    /// Resize the WebView manually. This is only required on Windows because its WebView API doesn't
    /// provide a way to resize automatically.
    fn resize(&self, new_size: WindowSize) -> Result<()>;

    fn inner_size(&self) -> WindowSize {
        self.window().inner_size()
    }

    fn version(&self) -> Result<String>;
    fn send_keyboard_input(&self, keyboard_input: KeyboardInput);
    fn send_mouse_position(&self, position: Vec2);
    fn send_mouse_event(&self, mouse_event: MouseEvent);
    fn get_texture(&mut self) -> Result<Option<Texture>>;
    fn tick_once(&mut self);

    fn tick(&mut self, tick_mode: TickMode) {
        match tick_mode {
            TickMode::Immediate => {
                self.tick_once();
            }

            TickMode::WaitFor(duration) => {
                let start = Instant::now();

                if start.elapsed() < duration {
                    std::thread::sleep(duration - start.elapsed());
                }

                // FIXME! takes time -> goes over tick duration
                self.tick_once();
            }

            TickMode::PeriodicWait(periodic_wait) => {
                let start = Instant::now();
                loop {
                    self.tick_once();
                    if start.elapsed() >= periodic_wait.duration {
                        return;
                    }

                    thread::sleep(periodic_wait.tick_interval);
                }
            }
        }
    }

    fn close(&mut self);
    fn load_html(&self, html: String);
    fn load_uri(&self, uri: String);
    fn reload(&self);
    fn set_is_visible(&mut self, is_visible: bool);
}

// Helper so all platforms handle RPC messages consistently.
pub fn rpc_proxy<T: HeadlessWindow>(
    window: &Rc<T>,
    js: String,
    handler: &dyn Fn(&T, RpcRequest) -> Option<RpcResponse>,
) -> Result<Option<String>> {
    let req = serde_json::from_str::<RpcRequest>(&js)
        .map_err(|e| Error::RpcScriptError(e.to_string(), js))?;

    let mut response = (handler)(window, req);
    // Got a synchronous response so convert it to a script to be evaluated
    if let Some(mut response) = response.take() {
        if let Some(id) = response.id {
            let js = if let Some(error) = response.error.take() {
                RpcResponse::get_error_script(id, error)?
            } else if let Some(result) = response.result.take() {
                RpcResponse::get_result_script(id, result)?
            } else {
                // No error or result, assume a positive response
                // with empty result (ACK)
                RpcResponse::get_result_script(id, Value::Null)?
            };
            Ok(Some(js))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

const RPC_VERSION: &str = "2.0";

/// RPC request message.
///
/// This usually passes to the [`RpcHandler`] or [`WindowRpcHandler`](crate::WindowRpcHandler) as
/// the parameter. You don't create this by yourself.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpcRequest {
    jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// RPC response message which being sent back to the Javascript side.
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    jsonrpc: String,
    pub(crate) id: Option<Value>,
    pub(crate) result: Option<Value>,
    pub(crate) error: Option<Value>,
}

impl RpcResponse {
    /// Create a new result response.
    pub fn new_result(id: Option<Value>, result: Option<Value>) -> Self {
        Self {
            jsonrpc: RPC_VERSION.to_string(),
            id,
            result,
            error: None,
        }
    }

    /// Create a new error response.
    pub fn new_error(id: Option<Value>, error: Option<Value>) -> Self {
        Self {
            jsonrpc: RPC_VERSION.to_string(),
            id,
            error,
            result: None,
        }
    }

    /// Get a script that resolves the promise with a result.
    pub fn get_result_script(id: Value, result: Value) -> Result<String> {
        let retval = serde_json::to_string(&result)?;
        Ok(format!("window.external.rpc._result({}, {})", id, retval))
    }

    /// Get a script that rejects the promise with an error.
    pub fn get_error_script(id: Value, result: Value) -> Result<String> {
        let retval = serde_json::to_string(&result)?;
        Ok(format!("window.external.rpc._error({}, {})", id, retval))
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}
