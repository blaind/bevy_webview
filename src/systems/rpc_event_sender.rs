use std::any::TypeId;

use bevy::{ecs::system::Resource, prelude::*};

use crate::{
    events::OutputEventMapping,
    types::{EventTransport, WebviewAction},
    Webview, WebviewEvent,
};

// Will send Bevy events of type `<T>` to webview
pub(crate) fn rpc_event_sender<T>(
    mut event_reader: EventReader<WebviewEvent<T>>,
    output_event_methods: Res<OutputEventMapping>,
    event_transport: Res<EventTransport>,
    webviews: Query<Entity, With<Webview>>,
) where
    T: Resource + serde::Serialize,
{
    if webviews.is_empty() {
        return;
    }

    for event in event_reader.read() {
        let event_key = output_event_methods.events.get(&TypeId::of::<T>()).unwrap();

        let payload = serde_json::to_string(&event.val).unwrap();

        event_transport
            .webview_action_tx
            .send(WebviewAction::SendOutputEvent(
                event.entity,
                event_key.to_string(),
                payload,
            ))
            .unwrap();
    }
}
