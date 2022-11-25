use bevy::log;

use bevy::prelude::Entity;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;

use crate::events::InputEvent;
use crate::types::{TextureReceivedEvent, WebviewAction};
use crate::WebviewCommand;

use headless_webview::prelude::*;
use headless_webview::types::{
    ElementState, KeyboardInput, MouseButton, MouseEvent, TickMode, WindowSize,
};

struct WebViewInner<T> {
    is_rpc_initialized: bool,
    webview: T,
}

/// Webview runner thread
///
/// Communicates through events
pub(crate) fn webview_runner_inner<T: HeadlessWindow>(
    texture_tx: Sender<TextureReceivedEvent>,
    webview_action_rx: Receiver<WebviewAction>,
    input_event_tx: Sender<InputEvent>,
    webview_implementation: fn() -> WindowBuilder<T>,
) {
    let mut webviews: HashMap<Entity, WebViewInner<<T as HeadlessWindow>::Webview>> =
        HashMap::new();

    for event in webview_action_rx.iter() {
        match event {
            WebviewAction::Launch(launch_event) => {
                log::debug!("Webview {:?}: launch webview instance", launch_event.entity);
                let window_builder = webview_implementation();
                let webview = launch_event.to_webview(window_builder, input_event_tx.clone());

                webviews.insert(
                    launch_event.entity,
                    WebViewInner {
                        is_rpc_initialized: false,
                        webview,
                    },
                );
            }

            WebviewAction::SetRPCInitialized(entity) => {
                log::debug!("Webview {:?}: set is_rpc_initialized", entity);

                if let Some(w) = webviews.get_mut(&entity) {
                    w.is_rpc_initialized = true;
                }
            }

            WebviewAction::MouseMotion((entity, position)) => {
                log::debug!("Webview {:?}: mouse motion={:?}", entity, position);
                if let Some(w) = webviews.get(&entity) {
                    w.webview.send_mouse_position(
                        // TODO: move position calc to the webview lib?
                        headless_webview::types::Vec2::new(
                            position.x * w.webview.window().width() as f32,
                            w.webview.window().height() as f32
                                - position.y * w.webview.window().height() as f32,
                        ),
                    );
                }
            }

            WebviewAction::Click((entity, button, element_state, position)) => {
                log::debug!(
                    "Webview {:?}: mouse={:?} state={:?} position={:?}",
                    entity,
                    button,
                    element_state,
                    position
                );

                if let Some(w) = webviews.get(&entity) {
                    w.webview.send_mouse_event(MouseEvent {
                        button: match button {
                            bevy::prelude::MouseButton::Left => MouseButton::Left,
                            bevy::prelude::MouseButton::Right => MouseButton::Right,
                            bevy::prelude::MouseButton::Middle => MouseButton::Middle,
                            bevy::prelude::MouseButton::Other(_) => continue,
                        },
                        state: match element_state {
                            bevy::input::ButtonState::Pressed => ElementState::Pressed,
                            bevy::input::ButtonState::Released => ElementState::Released,
                        },

                        // TODO: move position calc to the webview lib?
                        position: headless_webview::types::Vec2::new(
                            position.x * w.webview.window().width() as f32,
                            w.webview.window().height() as f32
                                - position.y * w.webview.window().height() as f32,
                        ),
                    });
                }
            }

            WebviewAction::Hover((_entity, _position)) => {}

            WebviewAction::TypeKeyboard((entity, keyboard_input)) => {
                log::debug!("Webview {:?}: keyboard event", entity);

                if let Some(w) = webviews.get(&entity) {
                    w.webview.send_keyboard_input(KeyboardInput {
                        state: match keyboard_input.state {
                            bevy::input::ButtonState::Pressed => ElementState::Pressed,
                            bevy::input::ButtonState::Released => ElementState::Released,
                        },
                    })
                }
            }

            WebviewAction::Resize((entity, size)) => {
                log::debug!("Webview {:?}: resized to {:?}", entity, size);

                if let Some(w) = webviews.get_mut(&entity) {
                    // TODO: this check can be removed after https://github.com/bevyengine/bevy/pull/3785 is merged
                    if w.webview.window().width() as f32 != size.x
                        || w.webview.window().height() as f32 != size.y
                    {
                        w.webview
                            .resize(WindowSize::new(size.x as u32, size.y as u32))
                            .unwrap();
                    }
                }
            }

            WebviewAction::Remove(entity) => {
                log::debug!("Webview {:?}: removed", entity);

                let _ = webviews.remove(&entity);
            }

            // Received RPC event, call Javascript in the webview
            WebviewAction::SendOutputEvent(entity, method, value) => {
                log::debug!(
                    "Webview {:?}: call RPC method {:?} ({} byte payload)",
                    entity,
                    method,
                    value.as_bytes().len()
                );

                let script = format!(
                    "window.external.rpc._message({:?}, JSON.parse({:?}));",
                    method, value
                );

                webviews
                    .iter()
                    .filter(filter_entity(entity))
                    .filter(|(_, w)| w.is_rpc_initialized)
                    .for_each(|(_, w)| {
                        w.webview.evaluate_script(&script).unwrap();
                    });
            }

            WebviewAction::Tick => {
                let mut texture_count = 0;

                for (entity, w) in webviews.iter_mut() {
                    w.webview.tick(TickMode::Immediate);

                    if let Ok(Some(texture)) = w.webview.get_texture() {
                        match texture_tx.send(TextureReceivedEvent {
                            entity: *entity,
                            texture,
                        }) {
                            Ok(_) => texture_count += 1,
                            Err(e) => log::warn!("Could not send webview texture: {:?}", e),
                        }
                    }
                }

                log::trace!("Webview(s) tick done, {} new textures", texture_count);
            }

            WebviewAction::AppExit => {
                log::debug!("Webview: AppExit");

                webviews.values_mut().for_each(|w| w.webview.close());

                break;
            }

            WebviewAction::RunCommand(entity, command) => {
                log::debug!("Webview ({:?}) command: {:?}", entity, command);

                let filtered_webviews = webviews.iter().filter(filter_entity(entity));

                match command {
                    WebviewCommand::LoadUri(uri) => {
                        filtered_webviews.for_each(|(_, w)| {
                            w.webview.load_uri(uri.clone());
                        });
                    }

                    WebviewCommand::LoadHtml(html) => {
                        filtered_webviews.for_each(|(_, w)| {
                            w.webview.load_html(html.clone());
                        });
                    }

                    WebviewCommand::Reload => {
                        filtered_webviews.for_each(|(_, w)| {
                            w.webview.reload();
                        });
                    }

                    WebviewCommand::RunJavascript(javascript) => {
                        filtered_webviews.for_each(|(_, w)| {
                            w.webview.evaluate_script(&javascript).unwrap();
                        });
                    }
                }
            }

            WebviewAction::SetVisibility(entity, is_visible) => {
                if let Some(w) = webviews.get_mut(&entity) {
                    w.webview.set_is_visible(is_visible);
                }
            }
        }
    }
}

fn filter_entity<T>(entity: Option<Entity>) -> impl FnMut(&(&Entity, T)) -> bool {
    move |(e, _)| match entity {
        Some(entity) => **e == entity,
        None => true,
    }
}
