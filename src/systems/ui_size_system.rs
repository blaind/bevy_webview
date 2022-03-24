use bevy::{log, prelude::*};

use crate::{
    types::{EventTransport, WebviewAction},
    Webview,
};

pub(crate) fn ui_size(
    event_transport: ResMut<EventTransport>,
    resized_webviews: Query<(Entity, &Node, &Webview), Changed<Node>>,
) {
    for (entity, node, _webview) in resized_webviews.iter() {
        log::debug!("Webview {:?} resized to {:?}", entity, node.size);

        event_transport
            .webview_action_tx
            .send(WebviewAction::Resize((entity, node.size)))
            .unwrap();
    }
}
