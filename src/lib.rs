//! Experimental webview integration for Bevy game engine for rapidly iterating and building UI's
//! using existing web-based technologies.
//!
//! # Example
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_webview::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(WebviewPlugin::new().register_engine(webview_engine::headless))
//!         .add_startup_system(setup);
//!         // .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(UiCameraBundle::default());
//!     commands.spawn_bundle(WebviewUIBundle {
//!         webview: Webview {
//!             uri: Some(String::from("http://bevyengine.org/")),
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     });
//! }
//! ```
use bevy::{
    prelude::*,
    ui::{widget::ImageMode, UiSystem},
};

pub mod prelude {
    pub use crate::{
        Webview, WebviewApp, WebviewBundle, WebviewCommand, WebviewEventReader, WebviewEventWriter,
        WebviewPlugin, WebviewSize, WebviewUIBundle,
    };

    pub use headless_webview::engines;

    #[cfg(feature = "engine")]
    pub use headless_webview_engine as webview_engine;
}

pub use serde;

mod events;
mod systems;
mod types;
mod webview;
use events::{
    BuiltinWebviewEvent, InputEvent, InputEventMapping, OutputEventMapping, WebviewEvent,
};
pub use events::{WebviewApp, WebviewEventReader, WebviewEventWriter};
use headless_webview::HeadlessWindow;
use headless_webview::WindowBuilder;
use serde::Serialize;
pub(crate) use systems::WebviewInteraction;
use webview::webview_thread;

pub(crate) const BUILTIN_RPC_INPUT_METHOD: &str = "_webview";

/// The webview plugin
///
/// Plugin configuration requires an engine registration
///
/// # Example
///
/// ```rust
/// # use bevy_webview::prelude::*;
/// let _ = WebviewPlugin::new().register_engine(webview_engine::headless);
/// ```
pub struct WebviewPlugin<ENGINE: HeadlessWindow> {
    pub(crate) engine: Option<fn() -> WindowBuilder<ENGINE>>,
}

impl<ENGINE: HeadlessWindow> WebviewPlugin<ENGINE> {
    pub fn new() -> Self {
        Self { engine: None }
    }
}

impl<ENGINE: HeadlessWindow> WebviewPlugin<ENGINE> {
    pub fn with_engine(engine: fn() -> WindowBuilder<ENGINE>) -> Self {
        Self {
            engine: Some(engine),
        }
    }

    pub fn register_engine(mut self, engine: fn() -> WindowBuilder<ENGINE>) -> Self {
        self.engine = Some(engine);
        self
    }
}

impl<ENGINE> Plugin for WebviewPlugin<ENGINE>
where
    ENGINE: HeadlessWindow + 'static,
{
    fn build(&self, app: &mut App) {
        let event_transport = webview_thread(WebviewEngine(
            self.engine
                .expect("Webview is missing an engine. Please use `.register_engine(engine)`"),
        ));

        app.insert_resource(InputEventMapping::default())
            .insert_resource(OutputEventMapping::default())
            .insert_resource(event_transport)
            .add_event::<InputEvent>()
            .add_event::<WebviewEvent<WebviewCommand>>()
            .add_webview_input_event::<BuiltinWebviewEvent>(BUILTIN_RPC_INPUT_METHOD)
            // PRE-SYSTEMS
            .add_system_to_stage(
                CoreStage::PreUpdate,
                systems::inject_rpc_requests_system.label(PreUpdateLabel::Pre),
            )
            // Systems
            .add_system(systems::rpc_builtin_event_handler)
            .add_system(systems::webview_ui_focus_system)
            .add_system(systems::ui_event)
            // PRE-PRE-POST updates - send events to webview
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::ui_size
                    .label(PostUpdateLabel::PrePre)
                    .after(UiSystem::Flex),
            )
            // PRE-POST updates - send events to webview
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::create_webview_system
                    .label(PostUpdateLabel::Pre)
                    .after(UiSystem::Flex),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::keyboard_event_system.label(PostUpdateLabel::Pre),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::webview_changed_system.label(PostUpdateLabel::Pre),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::rpc_command_system.label(PostUpdateLabel::Pre),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::removed_webviews_system.label(PostUpdateLabel::Pre),
            )
            // POST updates - tick the webviews
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::rpc_fallthrough_event_logger
                    .label(PostUpdateLabel::Update)
                    .after(PostUpdateLabel::Pre),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::tick_webviews_system
                    .label(PostUpdateLabel::Update)
                    .after(PostUpdateLabel::Pre),
            )
            // POST-POST updates - after tick, update textures
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::update_webview_textures
                    .label(PostUpdateLabel::Post)
                    .after(PostUpdateLabel::Update),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::app_exit
                    .label(PostUpdateLabel::Post)
                    .after(PostUpdateLabel::Update),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub(crate) enum PreUpdateLabel {
    Pre,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub(crate) enum PostUpdateLabel {
    PrePre,
    Pre,
    Update,
    Post,
}

/// Container for a webview engine
#[derive(Clone)]
pub(crate) struct WebviewEngine<T: HeadlessWindow>(fn() -> WindowBuilder<T>);

/// Webview bundle needed for creating a 3D webview. For UI/2D mode, see [`WebviewUIBundle`]
#[derive(Bundle, Clone, Default, Debug)]
pub struct WebviewBundle {
    /// WebView configuration
    pub webview: Webview,

    /// Size configuration
    pub size: WebviewSize,

    /// The transform of the node
    pub transform: Transform,

    /// The global transform of the node
    pub global_transform: GlobalTransform,

    /// Internal webview state, should not be edited directly
    pub webview_state: WebviewState,
}

/// Webview canvas size, used as a part of `WebviewBundle`. 2D/UI size is calculated automatically
#[derive(Component, Debug, Clone)]
pub struct WebviewSize {
    /// Width (in bevy world scale)
    pub x: f32,

    /// Height (in bevy world scale)
    pub y: f32,

    /// How many webview pixels should 1 unit of width/height have
    pub ppu: f32,
}

impl Default for WebviewSize {
    fn default() -> Self {
        Self {
            x: 1.,
            y: 1.,
            ppu: 400.,
        }
    }
}

impl WebviewSize {
    pub fn pixels_x(&self) -> u32 {
        (self.x * self.ppu).round() as u32
    }

    pub fn pixels_y(&self) -> u32 {
        (self.y * self.ppu).round() as u32
    }

    pub fn texture_byte_size_rgba_mb(&self) -> f32 {
        self.pixels_x() as f32 * self.pixels_y() as f32 * 4. / 1024. / 1024.
    }
}

/// Webview bundle needed for creating an UI webview. For 3D mode, see [`WebviewBundle`]
#[derive(Bundle, Clone, Default, Debug)]
pub struct WebviewUIBundle {
    /// WebView configuration
    pub webview: Webview,

    /// Describes the size of the node
    pub node: Node,

    /// Describes the style including flexbox settings
    pub style: Style,

    /// Configures how the image should scale
    pub image_mode: ImageMode,

    /// The calculated size based on the given image
    pub calculated_size: CalculatedSize,

    /// The transform of the node
    pub transform: Transform,

    /// The global transform of the node
    pub global_transform: GlobalTransform,

    /// Describes the visibility properties of the node
    pub visibility: Visibility,

    pub computed_visibility: ComputedVisibility,

    /// Interaction state
    pub interaction: WebviewInteraction,

    /// Internal webview state, should not be edited directly
    pub webview_state: WebviewState,
}

/// Webview [`Component`], should be inserted as a part of [`WebviewBundle`] or [`WebviewUIBundle`]
#[derive(Component, Clone, Debug)]
pub struct Webview {
    /// Load the provided URL
    ///
    /// Use `webview://` to load from bundled assets. Resolves to `/assets/webview/...`
    pub uri: Option<String>,

    /// Load the provided HTML string
    pub html: Option<String>,

    /// Color
    pub color: Color,

    /// Extra javascript that may be used for initialization (e.g. variable / state setup)
    pub initialization_script: Option<String>,
}

impl Default for Webview {
    fn default() -> Self {
        Self {
            uri: None,
            html: None,
            color: Default::default(),
            initialization_script: None,
        }
    }
}

/// Webview Commands for controlling a webview instance
///
/// Any future command added here should be available in the core API's:
/// * https://docs.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2?view=webview2-1.0.1072.54
/// * https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebView.html#webkit-web-view-reload
#[derive(Component, Serialize, Resource, Debug, Clone)]
pub enum WebviewCommand {
    /// Navigate to the given URI
    LoadUri(String),

    /// Load the given content string
    LoadHtml(String),

    /// Reloads the current contents of a webview (equal to `F5` press)
    Reload,

    /// Executes the given Javascript string
    RunJavascript(String),
}

/// Internal webview state, should not be edited directly
#[derive(Debug, Clone, Default, Component)]
pub struct WebviewState {
    pub(crate) texture_added: bool,
}
