/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::hashmap::HashMap;
use std::to_bytes::Cb;
use geom::size::Size2D;
use servo_msg::compositor_msg::Tile;

/// This is a struct used to store buffers when they are not in use.
/// The render task can quickly query for a particular size of buffer when it
/// needs it.
pub struct BufferMap<T> {
    /// A HashMap that stores the Buffers.
    map: HashMap<BufferKey, BufferValue<T>>,
    /// The current amount of memory stored by the BufferMap's buffers.
    mem: uint,
    /// The maximum allowed memory. Unused buffers willl be deleted
    /// when this threshold is exceeded.
    max_mem: uint,
    /// A monotonically increasing counter to track how recently tile sizes were used.
    counter: uint,
}

/// A key with which to store buffers. It is based on the size of the buffer.
struct BufferKey([uint, ..2]);

impl IterBytes for BufferKey {
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        let i = if lsb0 {0} else {1};
        self[i].iter_bytes(lsb0, |x| f(x)) && self[1 - i].iter_bytes(lsb0, |x| f(x))
    }
}

impl Eq for BufferKey {
    fn eq(&self, other: &BufferKey) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}

/// Create a key from a given size
impl BufferKey {
    fn get(input: Size2D<uint>) -> BufferKey {
        BufferKey([input.width, input.height])
    }
}

/// A helper struct to keep track of buffers in the HashMap
struct BufferValue<T> {
    /// An array of buffers, all the same size
    buffers: ~[T],
    /// The counter when this size was last requested
    last_action: uint,
}

impl<T: Tile> BufferMap<T> {
    // Creates a new BufferMap with a given buffer limit.
    pub fn new(max_mem: uint) -> BufferMap<T> {
        BufferMap {
            map: HashMap::new(),
            mem: 0u,
            max_mem: max_mem,
            counter: 0u,
        }
    }
    
    // Insert a new buffer into the map.
    pub fn insert(&mut self, new_buffer: T) {
        let new_key = BufferKey::get(new_buffer.get_size_2d());

        // If all our buffers are the same size and we're already at our
        // memory limit, no need to store this new buffer; just let it drop.
        if self.mem + new_buffer.get_mem() > self.max_mem && self.map.len() == 1 &&
            self.map.contains_key(&new_key) {
            return;
        }

        self.mem += new_buffer.get_mem();
        // use lazy insertion function to prevent unnecessary allocation
        self.map.find_or_insert_with(new_key, |_| BufferValue {
            buffers: ~[],
            last_action: self.counter
        }).buffers.push(new_buffer);

        let mut opt_key: Option<BufferKey> = None;
        while self.mem > self.max_mem {
            let old_key = match opt_key {
                Some(key) => key,
                None => {
                    match self.map.iter().min_by(|&(_, x)| x.last_action) {
                        Some((k, _)) => *k,
                        None => fail!("BufferMap: tried to delete with no elements in map"),
                    }
                }
            };
            if {
                let list = &mut self.map.get_mut(&old_key).buffers;
                self.mem -= list.pop().get_mem();
                list.is_empty()
            }
            { // then
                self.map.pop(&old_key); // Don't store empty vectors!
                opt_key = None;
            } else {
                opt_key = Some(old_key);
            }
        }
    }

    // Try to find a buffer for the given size.
    pub fn find(&mut self, size: Size2D<uint>) -> Option<T> {
        let mut flag = false; // True if key needs to be popped after retrieval.
        let key = BufferKey::get(size);
        let ret = match self.map.find_mut(&key) {
            Some(ref mut buffer_val) => {
                buffer_val.last_action = self.counter;
                self.counter += 1;
                
                let buffer = buffer_val.buffers.pop();
                self.mem -= buffer.get_mem();
                if buffer_val.buffers.is_empty() {
                    flag = true;
                }
                Some(buffer)
            }
            None => None,
        };
        
        if flag {
            self.map.pop(&key); // Don't store empty vectors!
        }

        ret
    }
}
