use bevy::log;
use bevy::prelude::Color;
use crossbeam_channel::Sender;
use headless_webview::http::{Request, Response, ResponseBuilder};
use headless_webview::webview::{RpcRequest, RpcResponse};
use std::fs::{canonicalize, read};
use std::path::PathBuf;

use crate::events::InputEvent;
use crate::types::LaunchEvent;

use headless_webview::prelude::*;

impl LaunchEvent {
    // Convert a launch event into a webview
    // This code actually creates the webview + window according to the engine implementation
    pub(crate) fn to_webview<T: HeadlessWindow>(
        &self,
        window_builder: WindowBuilder<T>,
        input_event_tx: Sender<InputEvent>,
    ) -> T::Webview {
        let mut window = window_builder;

        if self.webview.color.as_rgba().a() < 1.0 {
            window = window.with_transparent(true);
        }

        let window = window.with_inner_size(self.size.clone()).build().unwrap();

        let mut webview = WebviewBuilder::new(window).unwrap();

        webview = webview.with_color(match self.webview.color.as_rgba() {
            Color::Rgba {
                red,
                green,
                blue,
                alpha,
            } => headless_webview::webview::Color::new(red, green, blue, alpha),
            _ => unreachable!(),
        });

        if let Some(uri) = &self.webview.uri {
            webview = webview.with_url(uri).unwrap();

            webview = if uri.starts_with("webview://") {
                webview.with_custom_protocol("webview".into(), webview_request_handler)
            } else {
                webview
            }
        }

        if let Some(html) = &self.webview.html {
            webview = webview.with_html(html).unwrap()
        }

        if let Some(js) = &self.webview.initialization_script {
            webview = webview.with_initialization_script(&js);
        }

        let entity_clone = self.entity.clone();

        let webview = webview.with_rpc_handler(move |_window, request: RpcRequest| {
            log::trace!("Webview - RPC handler called");

            let id = request.id.clone();
            input_event_tx
                .send(InputEvent::new(entity_clone, request))
                .unwrap();

            Some(RpcResponse::new_result(
                id,
                Some(serde_json::Value::String(String::from("hello world!"))),
            ))
        });

        webview.build().unwrap()
    }
}

const ASSET_ROOT_PATH: &str = "assets/webview";

// Handle local asset requests
fn webview_request_handler(request: &Request) -> headless_webview::Result<Response> {
    let request_url = url::Url::parse(request.uri())?;

    let mut path = PathBuf::from(ASSET_ROOT_PATH);

    if let Some(segments) = request_url.path_segments() {
        for segment in segments {
            path = path.join(segment);
        }
    }

    log::debug!("Open webview:/// -path: {:?}", path);

    if !path.starts_with(ASSET_ROOT_PATH) {
        return ResponseBuilder::new()
            .status(401)
            .body("Directory traversal attempted! Denied".as_bytes().to_vec());
    }

    let path = canonicalize(&path)?;

    match mime_guess::from_path(&path).first() {
        Some(mime) => ResponseBuilder::new().mimetype(mime.essence_str()),
        None => ResponseBuilder::new().mimetype("application/octet-stream"),
    }
    .body(read(&path).map_err(|v| {
        if request_url.path() == "" {
            log::warn!(
                "Requested URL with empty path. Did you use webview:/// (with triple slash)?"
            );
        }
        v
    })?)
}
