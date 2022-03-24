use bevy::prelude::*;

use crate::{
    types::{EventTransport, WebviewAction},
    Webview,
};

pub(crate) fn tick_webviews_system(
    event_transport: ResMut<EventTransport>,
    webviews: Query<Entity, With<Webview>>,
) {
    if webviews.is_empty() {
        return;
    }

    event_transport
        .webview_action_tx
        .send(WebviewAction::Tick)
        .unwrap();
}
