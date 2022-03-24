use bevy::prelude::*;

use crate::types::{EventTransport, WebviewAction};

use super::WebviewInteraction;

pub(crate) fn ui_event(
    event_transport: ResMut<EventTransport>,
    interacted_webviews: Query<(Entity, &WebviewInteraction), Changed<WebviewInteraction>>,
) {
    for (entity, interaction) in interacted_webviews.iter() {
        match interaction {
            WebviewInteraction::Clicked(_) => (),
            /*
            WebviewInteraction::Clicked(offset) => event_transport
                .webview_action_tx
                .send(WebviewAction::Click((entity, *offset)))
                .unwrap(),
                */
            WebviewInteraction::Hovered(offset) => event_transport
                .webview_action_tx
                .send(WebviewAction::Hover((entity, *offset)))
                .unwrap(),

            WebviewInteraction::None => {}
        }
    }
}
