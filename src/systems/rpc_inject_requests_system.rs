use bevy::prelude::*;

use crate::{events::InputEvent, types::EventTransport};

/// Inject rpc events into bevy system as `InputEvents`'s (later converted into native events)
pub(crate) fn inject_rpc_requests_system(
    event_transport: Res<EventTransport>,
    mut input_events: EventWriter<InputEvent>,
) {
    for event in event_transport.input_event_rx.try_iter() {
        input_events.send(event);
    }
}
