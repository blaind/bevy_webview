use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::types::{EventTransport, WebviewAction};

use super::WebviewInteraction;

/// Maps bevy keyboard inputs to webview
///
/// Keyboard inputs are sent only when the webview component is being hovered or was clicked
pub(crate) fn keyboard_event_system(
    event_transport: ResMut<EventTransport>,
    webview_query: Query<(Entity, &WebviewInteraction)>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
) {
    for (entity, interaction) in webview_query.iter() {
        if let WebviewInteraction::Clicked(_) | WebviewInteraction::Hovered(_) = interaction {
            for keyboard_event in keyboard_input_events.iter() {
                event_transport
                    .webview_action_tx
                    .send(WebviewAction::TypeKeyboard((
                        entity,
                        keyboard_event.clone(),
                    )))
                    .unwrap();
            }
        }
    }
}
