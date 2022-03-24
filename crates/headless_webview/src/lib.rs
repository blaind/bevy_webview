#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

pub use crate::{
    http::{
        header::{InvalidHeaderName, InvalidHeaderValue},
        method::InvalidMethod,
        status::InvalidStatusCode,
        InvalidUri,
    },
    webview::{EngineWebview, WebviewBuilder},
    window::{HeadlessWindow, WindowBuilder},
};
use url::ParseError;

pub mod prelude {
    pub use crate::engines;
    pub use crate::types;
    pub use crate::{EngineWebview, HeadlessWindow, WebviewBuilder, WindowBuilder};
}

pub mod engines;
pub mod http;
pub mod types;
pub mod webview;
pub mod window;

/// Convenient type alias of Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Fail to fetch security manager")]
    MissingManager,
    #[error("Failed to initialize the script")]
    InitScriptError,
    #[error("Bad RPC request: {0} ((1))")]
    RpcScriptError(String, String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    UrlError(#[from] ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Duplicate custom protocol registered: {0}")]
    DuplicateCustomProtocol(String),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("Invalid uri: {0}")]
    InvalidUri(#[from] InvalidUri),
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(#[from] InvalidStatusCode),
    #[error("Invalid method: {0}")]
    InvalidMethod(#[from] InvalidMethod),
}
