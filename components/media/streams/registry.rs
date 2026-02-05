use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use uuid::Uuid;

use super::MediaStream;

type RegisteredMediaStream = Arc<Mutex<dyn MediaStream>>;

static MEDIA_STREAMS_REGISTRY: LazyLock<Mutex<HashMap<MediaStreamId, RegisteredMediaStream>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct MediaStreamId(Uuid);
impl MediaStreamId {
    pub fn new() -> MediaStreamId {
        Self { 0: Uuid::new_v4() }
    }

    pub fn id(self) -> Uuid {
        self.0
    }
}

pub fn register_stream(stream: Arc<Mutex<dyn MediaStream>>) -> MediaStreamId {
    let id = MediaStreamId::new();
    stream.lock().unwrap().set_id(id.clone());
    MEDIA_STREAMS_REGISTRY
        .lock()
        .unwrap()
        .insert(id.clone(), stream);
    id
}

pub fn unregister_stream(stream: &MediaStreamId) {
    MEDIA_STREAMS_REGISTRY.lock().unwrap().remove(stream);
}

pub fn get_stream(stream: &MediaStreamId) -> Option<Arc<Mutex<dyn MediaStream>>> {
    MEDIA_STREAMS_REGISTRY.lock().unwrap().get(stream).cloned()
}
