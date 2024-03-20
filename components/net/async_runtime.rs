use std::sync::Mutex;

use lazy_static::lazy_static;
use tokio::runtime::Runtime;

lazy_static! {
    pub static ref HANDLE: Mutex<Option<Runtime>> = Mutex::new(Some(Runtime::new().unwrap()));
}
