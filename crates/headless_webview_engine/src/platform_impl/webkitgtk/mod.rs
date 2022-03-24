use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, Once};

use gdk::gio::Cancellable;
use gdk::{
    ffi,
    glib::{translate::ToGlibPtr, Bytes},
    prelude::{Cast, WindowExtManual},
    EventButton, EventKey, EventMask, FromEvent,
};
use gdk::{EventMotion, WindowHints};

use gtk::{
    prelude::{ContainerExt, WidgetExt, WidgetExtManual},
    traits::{BoxExt, GtkWindowExt},
};

use headless_webview::types::{Vec2, WindowSize};
use headless_webview::webview::rpc_proxy;
use headless_webview::webview::web_context::WebContext;
use headless_webview::{Error, HeadlessWindow};

use webkit2gtk::traits::{SettingsExt, UserContentManagerExt, WebViewExt as webkit2gtkWebViewExt};
use webkit2gtk::{LoadEvent, UserContentInjectedFrames, UserScript, UserScriptInjectionTime};

use headless_webview::{
    types::{ElementState, KeyboardInput, MouseButton, MouseEvent, Texture, TextureFormat},
    webview::{EngineWebview, WebViewAttributes},
    window::{WindowAttributes, WindowBuilder},
    Result,
};

mod gtk_window;
mod web_context;
use web_context::GdkWebContext;

use self::gtk_window::GtkWindow;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        gtk::init().unwrap();
    });
}

pub fn windowed() -> WindowBuilder<GtkWindow<gtk::Window>> {
    init();

    let window: gtk::Window = gtk::Window::builder().build();
    let builder = WindowBuilder::new(window);
    builder.add_before_build_fn(windowed_prebuild)
}

fn windowed_prebuild(
    window: gtk::Window,
    attributes: WindowAttributes,
) -> (gtk::Window, WindowAttributes) {
    window.set_resizable(true);
    window.set_geometry_hints(
        Some(&window),
        Some(&gdk::Geometry::new(
            10,
            10,
            3840,
            2160,
            0,
            0,
            0,
            0,
            0f64,
            0f64,
            gdk::Gravity::Center,
        )),
        WindowHints::empty(),
    );

    window.set_title("bevy_webview");
    (window, attributes)
}

pub fn headless() -> WindowBuilder<GtkWindow<gtk::OffscreenWindow>> {
    init();

    let window: gtk::OffscreenWindow = gtk::OffscreenWindow::builder().build();
    let builder = WindowBuilder::new(window);
    builder.add_before_build_fn(headless_prebuild)
}

fn headless_prebuild(
    window: gtk::OffscreenWindow,
    attributes: WindowAttributes,
) -> (gtk::OffscreenWindow, WindowAttributes) {
    window.set_resizable(true);
    window.set_geometry_hints(
        Some(&window),
        Some(&gdk::Geometry::new(
            10,
            10,
            3840,
            2160,
            0,
            0,
            0,
            0,
            0f64,
            0f64,
            gdk::Gravity::Center,
        )),
        WindowHints::empty(),
    );

    (window, attributes)
}

pub struct GtkWebview<T: ContainerExt + WidgetExt + GtkWindowExt> {
    webview: Rc<webkit2gtk::WebView>,
    window: Rc<GtkWindow<T>>,
    webview_window: Rc<gdk::Window>,
    load_state: Rc<AtomicI32>, // castable to LoadState
    web_context: Rc<Mutex<WebContext<GdkWebContext>>>,
    is_visible: bool,
}

impl<T: ContainerExt + WidgetExt + GtkWindowExt> EngineWebview for GtkWebview<T> {
    type Window = GtkWindow<T>;
    type WebContext = GdkWebContext;

    fn new(
        window: Rc<Self::Window>,
        mut attributes: WebViewAttributes<Self::Window>,
        web_context: Option<Rc<Mutex<WebContext<Self::WebContext>>>>,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        log::trace!("webview::new() starting");

        // default_context allows us to create a scoped context on-demand
        let rc_mutex_web_context = match web_context {
            Some(w) => w,
            None => Rc::new(Mutex::new(Default::default())),
        };

        let mut web_context = rc_mutex_web_context.lock().unwrap();

        let webview: webkit2gtk::WebView = {
            let mut webview = webkit2gtk::WebViewBuilder::new();
            webview = webview.user_content_manager(web_context.os.manager());
            webview = webview.web_context(web_context.os.context());
            webview = webview.is_controlled_by_automation(web_context.os.allows_automation());
            webview.build()
        };

        web_context.os.register_automation(webview.clone());

        // Message handler
        let webview = Rc::new(webview);
        let wv = Rc::clone(&webview);
        let w = window.clone();
        let rpc_handler = attributes.rpc_handler.take();

        // Use the window hash as the script handler name
        let window_hash = {
            let mut hasher = DefaultHasher::new();
            w.id().hash(&mut hasher);
            hasher.finish().to_string()
        };

        let manager = web_context.os.manager();

        // Connect before registering as recommended by the docs
        manager.connect_script_message_received(None, move |_m, msg| {
            if let Some(js) = msg.js_value() {
                if let Some(rpc_handler) = &rpc_handler {
                    match rpc_proxy(&w, js.to_string(), rpc_handler) {
                        Ok(result) => {
                            let script = result.unwrap_or_default();
                            let cancellable: Option<&Cancellable> = None;
                            wv.run_javascript(&script, cancellable, |_| ());
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                }
            }
        });

        // Register the handler we just connected
        manager.register_script_message_handler(&window_hash);

        webview.add_events(
            EventMask::POINTER_MOTION_MASK
                | EventMask::BUTTON1_MOTION_MASK
                | EventMask::BUTTON_PRESS_MASK
                | EventMask::TOUCH_MASK,
        );

        let load_state = Rc::new(AtomicI32::new(LoadState::PreStart as i32));
        let inner_load_state = load_state.clone();

        webview.connect_load_changed(move |_, load_event| {
            log::trace!("Load event: {:?}", load_event);

            inner_load_state.store(
                match load_event {
                    LoadEvent::Started => LoadState::Started,
                    LoadEvent::Redirected => LoadState::Redirected,
                    LoadEvent::Committed => LoadState::Committed,
                    LoadEvent::Finished => LoadState::Finished,
                    LoadEvent::__Unknown(_) | _ => LoadState::Unknown,
                } as i32,
                Ordering::SeqCst,
            );
        });

        // Gtk application window can only contain one widget at a time.
        // In window, we add a GtkBox to pack menu bar. So we check if
        // there's a box widget here.
        if let Some(widget) = window.inner.children().pop() {
            let vbox = widget.downcast::<gtk::Box>().unwrap();
            // FIXME: safety?
            vbox.pack_start(&*webview, true, true, 0);
        }
        webview.grab_focus();

        // Enable webgl, webaudio, canvas features as default.
        if let Some(settings) = webkit2gtkWebViewExt::settings(&*webview) {
            settings.set_enable_webgl(true);
            settings.set_enable_webaudio(true);
            //settings.set_enable_accelerated_2d_canvas(true);

            settings.set_enable_write_console_messages_to_stdout(true); // FIXME: add setting

            if attributes.clipboard {
                settings.set_javascript_can_access_clipboard(true);
            }

            // Enable App cache
            settings.set_enable_offline_web_application_cache(true);
            settings.set_enable_page_cache(true);

            // Set user agent
            settings.set_user_agent(attributes.user_agent.as_deref());

            debug_assert_eq!(
                {
                    settings.set_enable_developer_extras(true);
                },
                ()
            );
        }

        // Color
        webview.set_background_color(&gdk::RGBA::new(
            attributes.color.r as f64,
            attributes.color.g as f64,
            attributes.color.b as f64,
            attributes.color.a as f64,
        ));

        window.inner.show_all();

        // Initialize message handler
        let mut init = String::with_capacity(67 + 20 + 20);
        init.push_str("window.external={invoke:function(x){window.webkit.messageHandlers[\"");
        init.push_str(&window_hash);
        init.push_str("\"].postMessage(x);}}");
        init_script(&webview, &init)?;

        // Initialize scripts
        for js in attributes.initialization_scripts {
            init_script(&webview, &js)?;
        }

        for (name, handler) in attributes.custom_protocols {
            match web_context.os.register_uri_scheme(&name, handler) {
                // Swallow duplicate scheme errors to preserve current behavior.
                // FIXME: we should log this error in the future
                Err(Error::DuplicateCustomProtocol(_)) => (),
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }

        drop(web_context);

        let gtk_webview = Self {
            webview_window: Rc::new(webview.window().unwrap()),
            webview,
            window,
            load_state,
            web_context: rc_mutex_web_context,
            is_visible: true,
        };

        // Navigation
        if let Some(url) = attributes.url {
            gtk_webview.load_uri(url.to_string());
        } else if let Some(html) = attributes.html {
            gtk_webview.load_html(html);
        }

        log::trace!(
            "webview::new() completed, webview version: {:?}",
            gtk_webview.version().unwrap()
        );

        Ok(gtk_webview)
    }

    fn load_uri(&self, uri: String) {
        let web_context = self.web_context.lock().unwrap();
        let parsed_url = url::Url::parse(&uri).unwrap();

        web_context
            .os
            .queue_load_uri(self.webview.clone(), parsed_url);

        web_context.os.flush_queue_loader();
    }

    fn load_html(&self, html: String) {
        self.webview.load_html(&html, Some("http://localhost"));
    }

    fn send_keyboard_input(&self, keyboard_input: KeyboardInput) {
        if !self.is_visible {
            return;
        }

        let mut event = gdk::Event::new(match keyboard_input.state {
            ElementState::Pressed => gdk::EventType::KeyPress,
            ElementState::Released => gdk::EventType::KeyRelease,
        });
        event.set_device(Some(&self.window.device));

        let mut event_key = <EventKey as FromEvent>::from(event).unwrap();
        let event_data = event_key.as_mut();

        event_data.window = self.webview_window.to_glib_full(); // FIXME: safety?
        event_data.send_event = 1;
        event_data.time = ffi::GDK_CURRENT_TIME as u32;
        event_data.state = 0; //ffi::KEYPRESS 0; // ffi::GDK_CONTROL_MASK;
        event_data.keyval = gdk::ffi::GDK_KEY_F5 as u32; // GDK_KEY_x
                                                         //gdk_key.length = 1; // ?
                                                         //gdk_key.string = ??? c_char

        event_data.hardware_keycode = 0; // ??
        event_data.group = 0;
        event_data.is_modifier = 0;
        event_key.put();
    }

    fn send_mouse_position(&self, position: Vec2) {
        if !self.is_visible {
            return;
        }

        let event = gdk::Event::new(gdk::EventType::MotionNotify);

        let mut event_motion = <EventMotion as FromEvent>::from(event).unwrap();
        let event_data = event_motion.as_mut();

        event_data.window = self.webview_window.to_glib_full(); // FIXME: safety?
        event_data.send_event = 1;
        event_data.time = ffi::GDK_CURRENT_TIME as u32;

        let area = self.webview.allocation();
        event_data.x = area.x() as f64 + position.x as f64;
        event_data.y = area.y() as f64 + position.y as f64;
        event_data.state = 0;

        event_data.device = self.window.device.to_glib_full(); // FIXME: safety?
        event_motion.put();
    }

    fn send_mouse_event(&self, mouse_event: MouseEvent) {
        if !self.is_visible {
            return;
        }

        let event = gdk::Event::new(match mouse_event.state {
            ElementState::Pressed => gdk::EventType::ButtonPress,
            ElementState::Released => gdk::EventType::ButtonRelease,
        });

        let mut event_button = <EventButton as FromEvent>::from(event).unwrap();
        let event_data = event_button.as_mut();

        event_data.window = self.webview_window.to_glib_full(); // FIXME: safety?
        event_data.send_event = 1;
        event_data.time = ffi::GDK_CURRENT_TIME as u32;

        let area = self.webview.allocation();
        event_data.x = area.x() as f64 + mouse_event.position.x as f64;
        event_data.y = area.y() as f64 + mouse_event.position.y as f64;
        event_data.state = 0; /*match mouse_event.button {
                                  MouseButton::Left => ffi::GDK_BUTTON1_MASK,
                                  MouseButton::Right => ffi::GDK_BUTTON2_MASK,
                                  MouseButton::Middle => ffi::GDK_BUTTON3_MASK,
                                  MouseButton::Other(_) => return,
                              };
                              */

        event_data.button = match mouse_event.button {
            MouseButton::Left => 1,
            MouseButton::Middle => 2,
            MouseButton::Right => 3,
            MouseButton::Other(value) => value as u32,
        };
        event_data.device = self.window.device.to_glib_full(); // FIXME: safety?
        event_button.put();
    }

    fn window(&self) -> &Self::Window {
        &self.window
    }

    fn get_texture(&mut self) -> Result<Option<Texture>> {
        if !self.window().has_events.load(Ordering::SeqCst)
            || LoadState::from_i32(self.load_state.load(Ordering::SeqCst)) == LoadState::PreStart
            || !self.is_visible
        {
            return Ok(None);
        }

        let gdk_window = self.window.inner.window().unwrap();

        let pixbuf = gdk_window
            .pixbuf(0, 0, gdk_window.width(), gdk_window.height())
            .unwrap();

        let pixbuf_bytes: Bytes = pixbuf.read_pixel_bytes().unwrap();

        let format = match pixbuf.n_channels() {
            3 => TextureFormat::Rgb8,
            4 => TextureFormat::Rgba8,
            _ => todo!(), // FIXME
        };

        self.window().has_events.store(false, Ordering::SeqCst);

        log::trace!(
            "Emitting texture (w={}, h={})",
            pixbuf.width(),
            pixbuf.height()
        );

        Ok(Some(Texture {
            width: pixbuf.width() as u32,
            height: pixbuf.height() as u32,
            format,
            data: pixbuf_bytes.to_vec(),
        }))
    }

    fn tick_once(&mut self) {
        // TODO is it okay not to process events? or maybe reduce the interval
        if !self.is_visible {
            return;
        }

        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }

    fn evaluate_script(&self, js: &str) -> Result<()> {
        let cancellable: Option<&Cancellable> = None;
        self.webview.run_javascript(js, cancellable, |_| ());
        Ok(())
    }

    fn resize(&self, new_size: WindowSize) -> Result<()> {
        log::trace!("resize to {:?}", new_size);

        self.window.resize(new_size)
    }

    fn close(&mut self) {
        // nothing to do?
    }

    fn version(&self) -> Result<String> {
        let (major, minor, patch) = unsafe {
            (
                webkit2gtk::ffi::webkit_get_major_version(),
                webkit2gtk::ffi::webkit_get_minor_version(),
                webkit2gtk::ffi::webkit_get_micro_version(),
            )
        };

        Ok(format!("webkit2gtk-v{}.{}.{}", major, minor, patch))
    }

    fn reload(&self) {
        self.webview.reload();
    }

    fn set_is_visible(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }
}

impl<T: ContainerExt + WidgetExt + GtkWindowExt> Drop for GtkWebview<T> {
    fn drop(&mut self) {
        // self.window().inner.window().unwrap().destroy();
        self.window().inner.close();
        self.tick_once();
    }
}

fn init_script(webview: &webkit2gtk::WebView, js: &str) -> Result<()> {
    if let Some(manager) = webview.user_content_manager() {
        let script = UserScript::new(
            js,
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        manager.add_script(&script);
    } else {
        return Err(Error::InitScriptError);
    }
    Ok(())
}

// Maps to webkit2gtk::LoadEvent
#[derive(PartialEq, Debug)]
pub enum LoadState {
    PreStart = -1,
    Started = 0,
    Redirected = 1,
    Committed = 2,
    Finished = 3,
    Unknown = 999,
}

impl LoadState {
    pub fn from_i32(val: i32) -> Self {
        match val {
            -1 => LoadState::PreStart,
            0 => LoadState::Started,
            1 => LoadState::Redirected,
            2 => LoadState::Committed,
            3 => LoadState::Finished,
            999 | _ => LoadState::Unknown,
        }
    }
}
