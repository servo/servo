/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use geom::size::Size2D;
use layers::platform::surface::NativePaintingGraphicsContext;
use layers::layers::LayerBuffer;
use std::hash::Hash;
use std::hash::sip::SipState;
use std::mem;

/// This is a struct used to store buffers when they are not in use.
/// The paint task can quickly query for a particular size of buffer when it
/// needs it.
pub struct BufferMap {
    /// A HashMap that stores the Buffers.
    map: HashMap<BufferKey, BufferValue>,
    /// The current amount of memory stored by the BufferMap's buffers.
    mem: uint,
    /// The maximum allowed memory. Unused buffers will be deleted
    /// when this threshold is exceeded.
    max_mem: uint,
    /// A monotonically increasing counter to track how recently tile sizes were used.
    counter: uint,
}

/// A key with which to store buffers. It is based on the size of the buffer.
#[deriving(Eq, Copy)]
struct BufferKey([uint, ..2]);

impl Hash for BufferKey {
    fn hash(&self, state: &mut SipState) {
        let BufferKey(ref bytes) = *self;
        bytes.as_slice().hash(state);
    }
}

impl PartialEq for BufferKey {
    fn eq(&self, other: &BufferKey) -> bool {
        let BufferKey(s) = *self;
        let BufferKey(o) = *other;
        s[0] == o[0] && s[1] == o[1]
    }
}

/// Create a key from a given size
impl BufferKey {
    fn get(input: Size2D<uint>) -> BufferKey {
        BufferKey([input.width, input.height])
    }
}

/// A helper struct to keep track of buffers in the HashMap
struct BufferValue {
    /// An array of buffers, all the same size
    buffers: Vec<Box<LayerBuffer>>,
    /// The counter when this size was last requested
    last_action: uint,
}

impl BufferMap {
    // Creates a new BufferMap with a given buffer limit.
    pub fn new(max_mem: uint) -> BufferMap {
        BufferMap {
            map: HashMap::new(),
            mem: 0u,
            max_mem: max_mem,
            counter: 0u,
        }
    }

    /// Insert a new buffer into the map.
    pub fn insert(&mut self, graphics_context: &NativePaintingGraphicsContext, new_buffer: Box<LayerBuffer>) {
        let new_key = BufferKey::get(new_buffer.get_size_2d());

        // If all our buffers are the same size and we're already at our
        // memory limit, no need to store this new buffer; just let it drop.
        if self.mem + new_buffer.get_mem() > self.max_mem && self.map.len() == 1 &&
            self.map.contains_key(&new_key) {
            new_buffer.destroy(graphics_context);
            return;
        }

        self.mem += new_buffer.get_mem();
        // use lazy insertion function to prevent unnecessary allocation
        let counter = &self.counter;
        match self.map.entry(new_key) {
            Occupied(entry) => {
                entry.into_mut().buffers.push(new_buffer);
            }
            Vacant(entry) => {
                entry.set(BufferValue {
                    buffers: vec!(new_buffer),
                    last_action: *counter,
                });
            }
        }

        let mut opt_key: Option<BufferKey> = None;
        while self.mem > self.max_mem {
            let old_key = match opt_key {
                Some(key) => key,
                None => {
                    match self.map.iter().min_by(|&(_, x)| x.last_action) {
                        Some((k, _)) => *k,
                        None => panic!("BufferMap: tried to delete with no elements in map"),
                    }
                }
            };
            if {
                let list = &mut self.map[old_key].buffers;
                let condemned_buffer = list.pop().take().unwrap();
                self.mem -= condemned_buffer.get_mem();
                condemned_buffer.destroy(graphics_context);
                list.is_empty()
            }
            { // then
                self.map.remove(&old_key); // Don't store empty vectors!
                opt_key = None;
            } else {
                opt_key = Some(old_key);
            }
        }
    }

    // Try to find a buffer for the given size.
    pub fn find(&mut self, size: Size2D<uint>) -> Option<Box<LayerBuffer>> {
        let mut flag = false; // True if key needs to be popped after retrieval.
        let key = BufferKey::get(size);
        let ret = match self.map.get_mut(&key) {
            Some(ref mut buffer_val) => {
                buffer_val.last_action = self.counter;
                self.counter += 1;

                let buffer = buffer_val.buffers.pop().take().unwrap();
                self.mem -= buffer.get_mem();
                if buffer_val.buffers.is_empty() {
                    flag = true;
                }
                Some(buffer)
            }
            None => None,
        };

        if flag {
            self.map.remove(&key); // Don't store empty vectors!
        }

        ret
    }

    /// Destroys all buffers.
    pub fn clear(&mut self, graphics_context: &NativePaintingGraphicsContext) {
        let map = mem::replace(&mut self.map, HashMap::new());
        for (_, value) in map.into_iter() {
            for tile in value.buffers.into_iter() {
                tile.destroy(graphics_context)
            }
        }
        self.mem = 0
    }
}
