use bevy::prelude::*;

use crate::{
    types::{EventTransport, WebviewAction},
    WebviewCommand, WebviewEvent,
};

pub(crate) fn rpc_command_system(
    mut webview_commands: EventReader<WebviewEvent<WebviewCommand>>,
    event_transport: Res<EventTransport>,
) {
    for command in webview_commands.read() {
        event_transport
            .webview_action_tx
            .send(WebviewAction::RunCommand(
                command.entity,
                command.val.clone(),
            ))
            .unwrap();
    }
}
