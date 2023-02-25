use std::collections::HashMap;

use bevy::{log, prelude::*};

use crate::{
    types::{EventTransport, WebviewAction},
    Webview, WebviewCommand,
};

pub(crate) fn webview_changed_system(
    changed_webviews: Query<(Entity, &Webview), Changed<Webview>>,
    webview_visibility_changes: Query<(Entity, &ComputedVisibility), (Changed<Visibility>, With<Webview>)>,
    mut previous_webviews: Local<HashMap<Entity, Webview>>,
    event_transport: Res<EventTransport>,
    removed_webviews: RemovedComponents<Webview>,
) {
    for (entity, webview) in changed_webviews.iter() {
        let previous = match previous_webviews.get(&entity) {
            Some(w) => w,
            None => {
                // skip new ones
                previous_webviews.insert(entity, webview.clone());
                continue;
            }
        };

        if webview.html.is_some() && webview.html != previous.html {
            log::debug!(
                "Webview {:?} HTML change from {:?} to {:?}",
                entity,
                previous.html,
                webview.html,
            );

            event_transport
                .webview_action_tx
                .send(WebviewAction::RunCommand(
                    Some(entity),
                    WebviewCommand::LoadHtml(webview.html.clone().unwrap()),
                ))
                .unwrap();
        }

        if webview.uri.is_some() && webview.uri != previous.uri {
            log::debug!(
                "Webview {:?} URI change from {:?} to {:?}",
                entity,
                previous.uri,
                webview.uri,
            );

            event_transport
                .webview_action_tx
                .send(WebviewAction::RunCommand(
                    Some(entity),
                    WebviewCommand::LoadUri(webview.uri.clone().unwrap()),
                ))
                .unwrap();
        }

        if webview.color != webview.color {
            log::warn!("Webview color changed programmatically - this has no effect, please recreate the webview");
        }
    }

    for (entity, visibility) in webview_visibility_changes.iter() {
        log::debug!(
            "Webview {:?} visibility change: {:?}",
            entity,
            visibility.is_visible()
        );

        event_transport
            .webview_action_tx
            .send(WebviewAction::SetVisibility(entity, visibility.is_visible()))
            .unwrap();
    }

    for entity in removed_webviews.iter() {
        previous_webviews.remove(&entity);
    }
}
