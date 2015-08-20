/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::size::Size2D;
use layers::platform::surface::{NativeDisplay, NativeSurface};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::hash::{Hash, Hasher};

/// This is a struct used to store surfaces when they are not in use.
/// The paint task can quickly query for a particular size of surface when it
/// needs it.
pub struct SurfaceMap {
    /// A HashMap that stores the Buffers.
    map: HashMap<SurfaceKey, SurfaceValue>,
    /// The current amount of memory stored by the SurfaceMap's surfaces.
    mem: usize,
    /// The maximum allowed memory. Unused surfaces will be deleted
    /// when this threshold is exceeded.
    max_mem: usize,
    /// A monotonically increasing counter to track how recently tile sizes were used.
    counter: usize,
}

/// A key with which to store surfaces. It is based on the size of the surface.
#[derive(Eq, Copy, Clone)]
struct SurfaceKey([i32; 2]);

impl Hash for SurfaceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let SurfaceKey(ref bytes) = *self;
        bytes.hash(state);
    }
}

impl PartialEq for SurfaceKey {
    fn eq(&self, other: &SurfaceKey) -> bool {
        let SurfaceKey(s) = *self;
        let SurfaceKey(o) = *other;
        s[0] == o[0] && s[1] == o[1]
    }
}

/// Create a key from a given size
impl SurfaceKey {
    fn get(input: Size2D<i32>) -> SurfaceKey {
        SurfaceKey([input.width, input.height])
    }
}

/// A helper struct to keep track of surfaces in the HashMap
struct SurfaceValue {
    /// An array of surfaces, all the same size
    surfaces: Vec<NativeSurface>,
    /// The counter when this size was last requested
    last_action: usize,
}

impl SurfaceMap {
    // Creates a new SurfaceMap with a given surface limit.
    pub fn new(max_mem: usize) -> SurfaceMap {
        SurfaceMap {
            map: HashMap::new(),
            mem: 0,
            max_mem: max_mem,
            counter: 0,
        }
    }

    pub fn insert_surfaces(&mut self, display: &NativeDisplay, surfaces: Vec<NativeSurface>) {
        for surface in surfaces {
            self.insert(display, surface);
        }
    }

    /// Insert a new buffer into the map.
    pub fn insert(&mut self, display: &NativeDisplay, mut new_surface: NativeSurface) {
        let new_key = SurfaceKey::get(new_surface.get_size());

        // If all our surfaces are the same size and we're already at our
        // memory limit, no need to store this new buffer; just let it drop.
        let new_total_memory_usage = self.mem + new_surface.get_memory_usage();
        if new_total_memory_usage > self.max_mem && self.map.len() == 1 &&
            self.map.contains_key(&new_key) {
            new_surface.destroy(display);
            return;
        }

        self.mem = new_total_memory_usage;
        new_surface.mark_wont_leak();

        // use lazy insertion function to prevent unnecessary allocation
        let counter = &self.counter;
        match self.map.entry(new_key) {
            Occupied(entry) => {
                entry.into_mut().surfaces.push(new_surface);
            }
            Vacant(entry) => {
                entry.insert(SurfaceValue {
                    surfaces: vec!(new_surface),
                    last_action: *counter,
                });
            }
        }

        let mut opt_key: Option<SurfaceKey> = None;
        while self.mem > self.max_mem {
            let old_key = match opt_key {
                Some(key) => key,
                None => {
                    match self.map.iter().min_by(|&(_, x)| x.last_action) {
                        Some((k, _)) => *k,
                        None => panic!("SurfaceMap: tried to delete with no elements in map"),
                    }
                }
            };
            if {
                let list = &mut self.map.get_mut(&old_key).unwrap().surfaces;
                let mut condemned_surface = list.pop().take().unwrap();
                self.mem -= condemned_surface.get_memory_usage();
                condemned_surface.destroy(display);
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
    pub fn find(&mut self, size: Size2D<i32>) -> Option<NativeSurface> {
        let mut flag = false; // True if key needs to be popped after retrieval.
        let key = SurfaceKey::get(size);
        let ret = match self.map.get_mut(&key) {
            Some(ref mut surface_val) => {
                surface_val.last_action = self.counter;
                self.counter += 1;

                let surface = surface_val.surfaces.pop().take().unwrap();
                self.mem -= surface.get_memory_usage();
                if surface_val.surfaces.is_empty() {
                    flag = true;
                }
                Some(surface)
            }
            None => None,
        };

        if flag {
            self.map.remove(&key); // Don't store empty vectors!
        }

        ret
    }

    pub fn mem(&self) -> usize {
        self.mem
    }
}
