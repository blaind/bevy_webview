use std::sync::atomic::Ordering;

use bevy::{log, prelude::EventReader};

use crate::events::InputEvent;

pub(crate) fn rpc_fallthrough_event_logger(mut input_events: EventReader<InputEvent>) {
    input_events
        .iter()
        .filter(|v| !v.matched.load(Ordering::SeqCst))
        .for_each(|event| 
            log::warn!(
                "Fallthrough event from Javascript. Register it by `.add_webview_input_event`. Requested method: {:?}",
                event.request.method
            )
        );
}
