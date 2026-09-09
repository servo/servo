use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericSender;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebLockThreadMsg {
    Query(GenericSender<LockManagerSnapshot>),
    Request(GenericSender<Lock>, String), // TODO: options
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockManagerSnapshot {
    held: Vec<LockInfo>,
    pending: Vec<LockInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockInfo {
    pub name: String,
    pub mode: LockMode,
    pub client_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LockMode {
    Exclusive,
    Shared,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Lock {
    pub name: String,
    pub mode: LockMode,
}
