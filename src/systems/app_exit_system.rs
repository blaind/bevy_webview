use bevy::{app::AppExit, prelude::*};

use crate::types::{EventTransport, WebviewAction};

pub(crate) fn app_exit(
    webview_messaging: ResMut<EventTransport>,
    mut app_exit_events: EventReader<AppExit>,
) {
    if let Some(_) = app_exit_events.read().last() {
        webview_messaging
            .webview_action_tx
            .send(WebviewAction::AppExit)
            .unwrap();
    }
}
