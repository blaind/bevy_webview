use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
};
use crossbeam_channel::{Receiver, Sender};
use headless_webview::types::{Texture, WindowSize};

use crate::{events::InputEvent, Webview, WebviewCommand};

#[derive(Debug)]
pub(crate) enum WebviewAction {
    /// Webview RPC on javascript-side was initialized
    SetRPCInitialized(Entity),
    /// Launch a new webview (open a web page in a window)
    Launch(LaunchEvent),
    /// Mouse motion over webview
    MouseMotion((Entity, Vec2)),
    /// Webview was clicked
    Click((Entity, MouseButton, ElementState, Vec2)),
    /// Webview was hovered
    Hover((Entity, Vec2)),
    /// Webview received keyboard input
    TypeKeyboard((Entity, KeyboardInput)),
    /// Webview should be resized
    Resize((Entity, Vec2)),
    /// Webview should be deleted
    Remove(Entity),
    /// Events to webview(s)
    SendOutputEvent(Option<Entity>, String, String),
    /// Tick webviews once (run event loop)
    Tick,
    /// AppExit event handling
    AppExit,
    /// Send a user command to webview(s)
    RunCommand(Option<Entity>, WebviewCommand),
    /// Visibility changes
    SetVisibility(Entity, bool),
}

/// Webview launch data
#[derive(Debug)]
pub(crate) struct LaunchEvent {
    pub entity: Entity,
    pub webview: Webview,
    pub size: WindowSize,
}

/// Texture from webview
#[derive(Debug)]
pub(crate) struct TextureReceivedEvent {
    pub entity: Entity,
    pub texture: Texture,
}

/// Takes care of event handling between webview impl and bevy system
pub(crate) struct EventTransport {
    pub webview_action_tx: Sender<WebviewAction>,
    pub texture_rx: Receiver<TextureReceivedEvent>,
    pub input_event_rx: Receiver<InputEvent>,
}
