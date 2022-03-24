use bevy::prelude::*;

use crate::{events::BuiltinWebviewEvent, types::EventTransport, WebviewEventReader};

pub(crate) fn rpc_builtin_event_handler(
    mut events: WebviewEventReader<BuiltinWebviewEvent>,
    mut commands: Commands,
    event_transport: Res<EventTransport>,
) {
    for (event, entity) in events.iter_with_entity() {
        match &event {
            BuiltinWebviewEvent::Despawn => {
                commands.entity(entity).despawn_recursive();
            }

            BuiltinWebviewEvent::Initialize => {
                event_transport
                    .webview_action_tx
                    .send(crate::types::WebviewAction::SetRPCInitialized(entity))
                    .unwrap();
            }
        }
    }
}
