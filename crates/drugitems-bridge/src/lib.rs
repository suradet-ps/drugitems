//! Invoke Tauri 2 backend commands from WASM.
//!
//! The Leptos frontend runs in a Tauri webview with `withGlobalTauri: true`,
//! which injects `window.__TAURI__.core.invoke`. This crate wraps that call
//! with serde serialization. On native targets it compiles to a stub so the
//! whole workspace stays `cargo check`-clean everywhere.

use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;
use zeroize::Zeroizing;

/// Errors raised while invoking a backend command.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// Command call failed on the backend (its error string).
    #[error("backend command error: {0}")]
    Command(String),
    /// Payload serialization failed.
    #[error("payload serialization failed: {0}")]
    Serialize(String),
    /// Not running inside a Tauri webview.
    #[error("drugitems-bridge is only available inside the Tauri webview (wasm32)")]
    NotWebView,
}

/// Result alias.
pub type Result<T> = std::result::Result<T, BridgeError>;

/// Invoke a Tauri command with a serde-serializable payload and decode the
/// JSON response into `T`.
///
/// The payload is serialized once; prefer [`invoke_serialized`] when the
/// payload contains a secret so the caller can drop the source value before
/// awaiting the backend.
pub async fn invoke<T: DeserializeOwned>(cmd: &str, args: impl Serialize) -> Result<T> {
    let args_json = Zeroizing::new(
        serde_json::to_string(&args).map_err(|e| BridgeError::Serialize(e.to_string()))?,
    );
    invoke_serialized(cmd, args_json).await
}

/// Invoke a Tauri command with an already-serialized JSON payload.
///
/// Secret-bearing callers serialize into a [`Zeroizing<String>`]; the buffer
/// is wiped here as soon as the webview has parsed it into a JS value, so the
/// secret is not held in wasm memory while the backend works.
pub async fn invoke_serialized<T: DeserializeOwned>(
    cmd: &str,
    args_json: Zeroizing<String>,
) -> Result<T> {
    imp::invoke_serialized(cmd, args_json).await
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::*;
    use js_sys::Promise;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], js_name = "invoke")]
        fn tauri_invoke(cmd: &str, args: JsValue) -> Promise;
    }

    /// Parse the pre-serialized payload, wipe it, then invoke.
    pub async fn invoke_serialized<T: DeserializeOwned>(
        cmd: &str,
        args_json: Zeroizing<String>,
    ) -> Result<T> {
        let args_value = js_sys::JSON::parse(&args_json)
            .map_err(|e| BridgeError::Serialize(format!("{e:?}")))?;
        // The wasm-side copy of the secret is no longer needed.
        drop(args_json);

        let promise = tauri_invoke(cmd, args_value);
        let value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| {
                // The rejection value is the backend's serialized error (a typed
                // `CommandError` JSON object) - hand it back as raw JSON text so
                // the caller can deserialize it.
                let json = js_sys::JSON::stringify(&e)
                    .ok()
                    .and_then(|j| j.as_string())
                    .unwrap_or_else(|| format!("{e:?}"));
                BridgeError::Command(json)
            })?;

        let json = js_sys::JSON::stringify(&value)
            .map_err(|e| BridgeError::Serialize(format!("{e:?}")))?;
        let text = json.as_string().unwrap_or_else(|| "null".to_string());
        serde_json::from_str(&text).map_err(|e| BridgeError::Serialize(e.to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::*;

    /// Stub used when compiling for native targets.
    pub async fn invoke_serialized<T: DeserializeOwned>(
        _cmd: &str,
        _args_json: Zeroizing<String>,
    ) -> Result<T> {
        Err(BridgeError::NotWebView)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Ping {
        ok: bool,
    }

    #[tokio::test]
    async fn native_stub_returns_not_webview() {
        let r = invoke::<Ping>("ping", serde_json::json!({})).await;
        assert!(matches!(r, Err(BridgeError::NotWebView)));
    }

    #[tokio::test]
    async fn native_stub_accepts_pre_serialized_payload() {
        let payload = Zeroizing::new("{\"secret\":\"s3cret\"}".to_string());
        let r = invoke_serialized::<Ping>("ping", payload).await;
        assert!(matches!(r, Err(BridgeError::NotWebView)));
    }
}
