use std::{any::TypeId, collections::HashMap, sync::atomic::AtomicBool};

use bevy::{
    ecs::system::{Resource, SystemParam},
    prelude::*,
};
use headless_webview::webview::RpcRequest;
use serde::{Deserialize, Serialize};

use crate::{systems, PostUpdateLabel, PreUpdateLabel};

/// Mapping of RPC Input Event methods
#[derive(Default, Resource)]
pub(crate) struct InputEventMapping {
    pub events: HashMap<TypeId, &'static str>,
}

/// Mapping of RPC Output Event methods
#[derive(Default, Resource)]
pub(crate) struct OutputEventMapping {
    pub events: HashMap<TypeId, &'static str>,
}

// RPC events from a webview
pub(crate) struct InputEvent {
    // Sending entity
    pub entity: Entity,

    // Barebones request to be converted into Bevy-world
    pub request: RpcRequest,

    // Whether the request was converted to native event, or it fell through
    // Fell through = was not handled by any converter
    pub matched: AtomicBool,
}

impl InputEvent {
    pub fn new(entity: Entity, request: RpcRequest) -> Self {
        Self {
            matched: AtomicBool::new(false),
            entity,
            request,
        }
    }
}

/// Wraps an event of type `T` into a webview structure
#[derive(Deserialize, Serialize, Debug)]
pub struct WebviewEvent<T> {
    pub(crate) entity: Option<Entity>,
    pub(crate) val: T,
}

impl<T> WebviewEvent<T> {
    pub fn new(entity: Option<Entity>, val: T) -> Self {
        Self { entity, val }
    }
}

/// Read Events sent from Javascript
#[derive(SystemParam)]
pub struct WebviewEventReader<'w, 's, T: Resource> {
    pub events: EventReader<'w, 's, WebviewEvent<T>>,
}

impl<'w, 's, T: Resource> WebviewEventReader<'w, 's, T> {
    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = &T> {
        self.events
            .iter_with_id()
            .map(|(event, _id)| event)
            .map(|event| &event.val)
    }

    pub fn iter_with_entity(&mut self) -> impl DoubleEndedIterator<Item = (&T, Entity)> {
        self.events
            .iter_with_id()
            .map(|(event, _id)| event)
            .map(|event| (&event.val, event.entity.unwrap()))
    }

    // #[inline]
    // pub fn len(&self) -> usize {
    //     self.events.len()
    // }

    // #[inline]
    // pub fn is_empty(&self) -> bool {
    //     self.events.is_empty()
    // }
}

/// Send Events to webview Javascript
#[derive(SystemParam)]
pub struct WebviewEventWriter<'w, 's, T: Resource> {
    pub events: EventWriter<'w, 's, WebviewEvent<T>>,
}

impl<'w, 's, T: Resource> WebviewEventWriter<'w, 's, T> {
    /// Will send an event to **all** webviews
    pub fn send(&mut self, event: T) {
        self.events.send(WebviewEvent::new(None, event));
    }

    /// Will send an event to a specific webview, identified by `Entity`
    pub fn send_to_entity(&mut self, entity: Entity, event: T) {
        self.events.send(WebviewEvent::new(Some(entity), event));
    }
}

/// Trait that extends a Bevy [`App`] for registring webview events
/// # Example
///
/// ```rust
/// # use bevy_webview::serde::{Serialize, Deserialize};
/// use bevy::prelude::*;
/// use bevy_webview::prelude::*;
///
/// #[derive(Deserialize, Debug)]
/// pub struct LoginRequest {
///     username: String,
/// }
///
/// #[derive(Serialize, Debug)]
/// pub struct AppTime {
///     seconds_since_startup: f64,
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(WebviewPlugin::new().register_engine(webview_engine::headless))
///         .add_webview_input_event::<LoginRequest>("login")
///         .add_webview_output_event::<AppTime>("app_time")
///         .add_system(login_handler)
///         .add_system(send_time_system);
///         // .run();
/// }
///
/// fn login_handler(mut login_request_events: WebviewEventReader<LoginRequest>) {
///     for event in login_request_events.iter() {
///         println!("Received a login request, username={:?}", event.username);
///     }
/// }
///
/// fn send_time_system(mut app_time: WebviewEventWriter<AppTime>, time: Res<Time>) {
///     app_time.send(AppTime {
///         seconds_since_startup: time.seconds_since_startup(),
///     });
/// }
/// ```
pub trait WebviewApp {
    /// Register an input webview event. `method` is an identifier key for sending messages from JS
    fn add_webview_input_event<T>(&mut self, method: &'static str) -> &mut Self
    where
        T: Resource + for<'de> serde::Deserialize<'de>;

    /// Register an output webview event. `method` is an identifier key for hooking into events from JS
    fn add_webview_output_event<T>(&mut self, method: &'static str) -> &mut Self
    where
        T: Resource + for<'de> serde::Serialize;
}

impl WebviewApp for App {
    fn add_webview_input_event<T>(&mut self, method: &'static str) -> &mut Self
    where
        T: Resource + for<'de> serde::Deserialize<'de>,
    {
        let mut rpc_input_events = self
            .world
            .get_resource_mut::<InputEventMapping>()
            .expect("Add `WebviewPlugin` before calling `.add_webview_input_event`");

        rpc_input_events.events.insert(TypeId::of::<T>(), method);

        self.add_event::<WebviewEvent<T>>();

        self.add_system_to_stage(
            CoreStage::PreUpdate,
            systems::rpc_event_receiver::<T>.after(PreUpdateLabel::Pre),
        );
        self
    }

    fn add_webview_output_event<T>(&mut self, method: &'static str) -> &mut Self
    where
        T: Resource + for<'de> serde::Serialize,
    {
        let mut rpc_output_events = self
            .world
            .get_resource_mut::<OutputEventMapping>()
            .expect("Add `WebviewPlugin` before calling `.add_webview_input_event`");

        rpc_output_events.events.insert(TypeId::of::<T>(), method);

        self.add_event::<WebviewEvent<T>>();

        self.add_system_to_stage(
            CoreStage::PostUpdate,
            systems::rpc_event_sender::<T>.label(PostUpdateLabel::Pre),
        );

        self
    }
}

/// Enum of builtin event methods
#[derive(Deserialize, Debug, Resource)]
#[serde(rename_all(deserialize = "lowercase"))]
pub(crate) enum BuiltinWebviewEvent {
    Despawn,
    Initialize,
}
