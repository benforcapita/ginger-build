// Ginger Code — Msgpack-RPC bridge for Neovim
// Handles communication with nvim --embed via msgpack-rpc protocol.

use rmpv::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("rpc read error: {0}")]
    Read(String),
    #[error("rpc write error: {0}")]
    Write(String),
    #[error("rpc decode error: {0}")]
    Decode(String),
    #[error("rpc timeout")]
    Timeout,
}

/// Msgpack-RPC message types
#[derive(Debug, Clone)]
pub enum RpcMessage {
    Request { msgid: u64, method: String, params: Vec<Value> },
    Response { msgid: u64, result: Value, error: Value },
    Notification { method: String, params: Vec<Value> },
}

/// Decode a msgpack-rpc message from a raw Value array.
pub fn decode_message(val: &Value) -> Result<RpcMessage, RpcError> {
    let arr = val.as_array().ok_or_else(|| RpcError::Decode("not an array".into()))?;
    if arr.is_empty() {
        return Err(RpcError::Decode("empty message".into()));
    }

    let msgtype = arr[0].as_i64().ok_or_else(|| RpcError::Decode("invalid msgtype".into()))?;

    match msgtype {
        0 => {
            // Request: [0, msgid, method, params]
            if arr.len() < 4 {
                return Err(RpcError::Decode("request too short".into()));
            }
            let msgid = arr[1].as_u64().unwrap_or(0);
            let method = arr[2].as_str().unwrap_or("").to_string();
            let params = arr[3].as_array().cloned().unwrap_or_default();
            Ok(RpcMessage::Request { msgid, method, params })
        }
        1 => {
            // Response: [1, msgid, error, result]
            if arr.len() < 4 {
                return Err(RpcError::Decode("response too short".into()));
            }
            let msgid = arr[1].as_u64().unwrap_or(0);
            let error = arr[2].clone();
            let result = arr[3].clone();
            Ok(RpcMessage::Response { msgid, result, error })
        }
        2 => {
            // Notification: [2, method, params]
            if arr.len() < 3 {
                return Err(RpcError::Decode("notification too short".into()));
            }
            let method = arr[1].as_str().unwrap_or("").to_string();
            let params = arr[2].as_array().cloned().unwrap_or_default();
            Ok(RpcMessage::Notification { method, params })
        }
        _ => Err(RpcError::Decode(format!("unknown msgtype: {msgtype}"))),
    }
}

/// Encode a msgpack-rpc request.
pub fn encode_request(msgid: u64, method: &str, params: Vec<Value>) -> Value {
    Value::Array(vec![
        Value::Integer(0.into()),
        Value::Integer(msgid.into()),
        Value::String(method.into()),
        Value::Array(params),
    ])
}

/// Encode a msgpack-rpc notification.
pub fn encode_notification(method: &str, params: Vec<Value>) -> Value {
    Value::Array(vec![
        Value::Integer(2.into()),
        Value::String(method.into()),
        Value::Array(params),
    ])
}

/// Encode a msgpack-rpc response.
pub fn encode_response(msgid: u64, result: Value, error: Value) -> Value {
    Value::Array(vec![
        Value::Integer(1.into()),
        Value::Integer(msgid.into()),
        error,
        result,
    ])
}

/// RPC client state — tracks next msgid and pending requests.
pub struct RpcClient {
    next_msgid: Arc<Mutex<u64>>,
}

impl RpcClient {
    pub fn new() -> Self {
        Self {
            next_msgid: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn next_msgid(&self) -> u64 {
        let mut id = self.next_msgid.lock().await;
        let current = *id;
        *id += 1;
        current
    }
}

impl Default for RpcClient {
    fn default() -> Self { Self::new() }
}