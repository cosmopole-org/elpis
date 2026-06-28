//! The host-call envelope and reply helpers shared with the `elpian-vm`.
//!
//! When guest JS calls `askHost(apiName, ...args)` the VM pauses and hands the
//! embedder a JSON envelope `{"machineId","apiName","payload"}`, where `payload`
//! is the **raw JSON array** of the call arguments. The embedder services the
//! call and resumes the VM with a plain-JSON reply value (the VM's
//! `continue_run` accepts a bare JSON value as the call's return).
//!
//! This mirrors the `elpa-protocol::HostCall` design.

use serde::Deserialize;
use serde_json::Value;

/// Parsed `askHost` envelope.
#[derive(Debug, Clone, Default)]
pub struct HostCall {
    pub machine_id: String,
    pub api_name: String,
    /// Raw JSON text of the argument array; parse per `api_name`.
    pub payload: String,
}

impl HostCall {
    /// Parse the envelope JSON the VM hands back. Accepts both the raw-inline
    /// payload (common) and the legacy escaped-string payload.
    pub fn parse(json: &str) -> Result<HostCall, serde_json::Error> {
        let mut v: Value = serde_json::from_str(json)?;
        let machine_id = take_string(&mut v, "machineId");
        let api_name = take_string(&mut v, "apiName");
        let payload = match v.get_mut("payload").map(|p| p.take()) {
            Some(Value::String(s)) => s,
            Some(other) => other.to_string(),
            None => String::new(),
        };
        Ok(HostCall { machine_id, api_name, payload })
    }

    /// Parse the argument array.
    pub fn args(&self) -> Vec<Value> {
        match serde_json::from_str::<Value>(&self.payload) {
            Ok(Value::Array(a)) => a,
            Ok(other) => vec![other],
            Err(_) => Vec::new(),
        }
    }

    /// The `n`th call argument, if present.
    pub fn arg(&self, n: usize) -> Option<Value> {
        self.args().into_iter().nth(n)
    }

    /// The first argument deserialized into `T`.
    pub fn arg_as<T: for<'de> Deserialize<'de>>(&self, n: usize) -> Option<T> {
        self.arg(n).and_then(|v| serde_json::from_value(v).ok())
    }
}

fn take_string(v: &mut Value, key: &str) -> String {
    match v.get_mut(key).map(|p| p.take()) {
        Some(Value::String(s)) => s,
        _ => String::new(),
    }
}

/// The reply the embedder resumes the VM with. The VM injects this as the
/// `askHost` call's return value.
pub mod reply {
    use serde_json::{json, Value};

    /// `null` / `undefined` return.
    pub fn null() -> String {
        "null".to_string()
    }

    /// A successful, value-less acknowledgement: `{"ok": true}`.
    pub fn ok() -> String {
        json!({ "ok": true }).to_string()
    }

    /// An error reply: `{"ok": false, "error": msg}`.
    pub fn err(msg: &str) -> String {
        json!({ "ok": false, "error": msg }).to_string()
    }

    /// Wrap an arbitrary JSON value as the return value.
    pub fn value(v: Value) -> String {
        v.to_string()
    }

    /// A successful reply carrying a payload: `{"ok": true, "data": data}`.
    pub fn data(data: Value) -> String {
        json!({ "ok": true, "data": data }).to_string()
    }
}
