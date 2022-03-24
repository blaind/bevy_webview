use bevy::prelude::*;

use crate::{
    types::{EventTransport, WebviewAction},
    Webview,
};

/// This system takes care of initialing required `PbrBundle` for the webview
pub(crate) fn removed_webviews_system(
    event_transport: ResMut<EventTransport>,
    removed_webviews: RemovedComponents<Webview>,
) {
    for entity in removed_webviews.iter() {
        event_transport
            .webview_action_tx
            .send(WebviewAction::Remove(entity))
            .unwrap();
    }
}
