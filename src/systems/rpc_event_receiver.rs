use std::{any::TypeId, sync::atomic::Ordering};

use bevy::{ecs::system::Resource, log, prelude::*};

use crate::{
    events::{InputEvent, InputEventMapping},
    WebviewEvent,
};

pub(crate) fn rpc_event_receiver<T>(
    mut input_events: EventReader<InputEvent>,
    mut output_event_writer: EventWriter<WebviewEvent<T>>,
    input_event_methods: Res<InputEventMapping>,
) where
    T: Resource + for<'de> serde::Deserialize<'de>,
{
    for event in input_events
        .read()
        .filter(|v| {
            let expected = Some(v.request.method.as_str());
            let matched = input_event_methods
                .events
                .get(&TypeId::of::<T>())
                .map(|v| *v);
            expected == matched
        })
        .filter(|v| v.request.params.is_some())
    {
        let val: Result<T, serde_json::Error> =
            serde_json::from_value(event.request.params.as_ref().unwrap()[0].clone());

        event.matched.store(true, Ordering::SeqCst);

        match val {
            Ok(val) => {
                log::debug!("Received incoming event method={:?}", event.request.method);
                output_event_writer.send(WebviewEvent::new(Some(event.entity), val))
            }

            Err(e) => {
                let method = &event.request.method;
                let type_name = std::any::type_name::<WebviewEvent<T>>();

                log::warn!(
                    "method={:?} (type={:?}) deserialization error: {}",
                    method,
                    type_name,
                    e.to_string()
                );
            }
        }
    }
}
