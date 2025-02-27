/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;
use std::thread::{self, Builder};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use libc::c_void;
use parking_lot::RwLock;
use pixels::ImageContainer;
use script_layout_interface::LayoutImageAnimateHelper;
use script_traits::UntrustedNodeAddress;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use style::dom::OpaqueNode;

use crate::dom::node::from_untrusted_node_address;
use crate::task_source::SendableTaskSource;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ImageAnimationMessage {
    Active,
}

type ActiveKey = OpaqueNode; // Each node should have only one entry.

pub struct ImageAnimationSet {
    //use Opaque Node ID, URL, Image Key as identifier? for our active frame tracking.
    store: RwLock<HashMap<ActiveKey, ImageAnimateState>>, // storing the active frame id of each image.
}

pub struct ImageAnimationManager {
    pub image_animation_set: Arc<ImageAnimationSet>, // Need a IPC Channel to handle GraceFul ShutDown.
}

pub struct ImageAnimateState {
    pub frame_length: usize,
    pub frames_duration: Vec<Duration>,
    pub current_active_index: usize,
    pub last_update_time: f64,       // secs f64
    pub image_url: Option<ServoUrl>, // Potentially keeping it.
}

impl ImageAnimateState {
    pub fn next_frame(&mut self, new_time_value: f64) {
        self.last_update_time = new_time_value;
        self.current_active_index = (self.current_active_index + 1) % self.frame_length;
    }
}

impl ImageAnimationManager {
    pub fn new() -> ImageAnimationManager {
        ImageAnimationManager {
            image_animation_set: Arc::new(ImageAnimationSet::new()),
        }
    }

    pub fn start(&self, task_source: SendableTaskSource) {
        let store = self.image_animation_set.clone();
        //let (ipc_sender, ipc_receiver) = ipc::channel().unwrap();

        Builder::new()
            .name("ImageAnimation".to_string())
            .spawn(move || {
                // (Ray)TODO: Should change the type constraint if we need to update the core.
                loop {
                    //TODO: Set the exit condition.
                    thread::sleep(Duration::from_millis(10));

                    let inner_store = store.clone();
                    task_source.queue(task!(handle_image_animation: move ||{
                        inner_store.update_frame_with_new_timeline_value(Self::get_current_time());
                    }));
                }
            })
            .unwrap();
    }

    fn get_current_time() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }
}

impl ImageAnimationSet {
    pub fn new() -> Self {
        ImageAnimationSet {
            store: Default::default(),
        }
    }

    pub fn to_layout_helper(&self) -> LayoutImageAnimateHelper {
        let mut map = HashMap::new();
        self.store.read().iter().for_each(|(node, state)| {
            map.insert(*node, state.current_active_index);
        });
        LayoutImageAnimateHelper { mapping: map }
    }

    #[allow(unsafe_code)]
    pub fn update_frame_with_new_timeline_value(&self, new_time_value: f64) {
        // is new_time_value some kind of duration???
        // 1. iterate through the hashmap, get the opaque node of those who need to update frame.
        //

        self.store
            .write()
            .iter_mut()
            .filter(|(_node, state)| {
                let current_frame_exist_duration = new_time_value - state.last_update_time;
                current_frame_exist_duration >=
                    state.frames_duration[state.current_active_index].as_secs_f64()
            })
            .map(|(node, state)| {
                state.next_frame(new_time_value);
                node
            })
            .for_each(|node| {
                // 2. set node dirty for those who need update.
                unsafe {
                    let address = UntrustedNodeAddress(node.0 as *const c_void);
                    let node = from_untrusted_node_address(address);
                    node.dirty(crate::dom::node::NodeDamage::NodeStyleDamaged);
                }
            });
    }
    pub fn register_animation(
        &self,
        image: Arc<ImageContainer>,
        key: ActiveKey,
        url: Option<ServoUrl>,
    ) {
        //(Ray)TODO: potentially will fail, consider adding result here to better handle error

        let initial_state = ImageAnimateState {
            frame_length: image.frames.len(),
            frames_duration: image.frames.iter().map(|f| f.delay.unwrap()).collect(),
            current_active_index: 0,
            last_update_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(), // take reference from animation how they handle time value?
            image_url: url.clone(),
        };
        self.store.write().insert(key, initial_state);
    }
    pub fn cancel_animation(&self, key: &ActiveKey) {
        self.store.write().remove(key);
    }
}
