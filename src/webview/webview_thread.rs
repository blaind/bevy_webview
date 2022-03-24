use std::thread;

use crossbeam_channel::unbounded;
use headless_webview::HeadlessWindow;

use crate::types::EventTransport;
use crate::webview::webview_runner_inner;
use crate::WebviewEngine;

/// This acts as a communication bridge between webview implementation and bevy systems
pub(crate) fn webview_thread<T: 'static + HeadlessWindow>(
    webview_implementation: WebviewEngine<T>,
) -> EventTransport {
    let (webview_action_tx, webview_action_rx) = unbounded();
    let (texture_tx, texture_rx) = unbounded();
    let (input_event_tx, input_event_rx) = unbounded();

    let impl_fn = webview_implementation.0.clone();

    thread::Builder::new()
        .name("webview_runner".to_string())
        .spawn(move || {
            // create an empty window builder / do a lazy init for the engine implementation
            // happens in thread, speeds up first webview launch later on
            let _ = (impl_fn)();

            // start runner
            webview_runner_inner(texture_tx, webview_action_rx, input_event_tx, impl_fn);
        })
        .unwrap();

    EventTransport {
        webview_action_tx,
        texture_rx,
        input_event_rx,
    }
}
