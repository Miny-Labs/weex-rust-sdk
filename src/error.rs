use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeexError {
    #[error("HTTP Request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API Error: code={code}, msg={msg}")]
    Api { code: String, msg: String },

    #[error("Serialization Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Signing Error: {0}")]
    Signing(String),

    #[error("Unknown Error: {0}")]
    Unknown(String),
}
