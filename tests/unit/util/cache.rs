/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use util::cache::{HashCache, LRUCache};

#[test]
fn test_hashcache() {
    let mut cache: HashCache<usize, Cell<&str>> = HashCache::new();

    cache.insert(1, Cell::new("one"));
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_none());

    cache.find_or_create(&2, |_v| { Cell::new("two") });
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_some());
}

#[test]
fn test_lru_cache() {
    let one = Cell::new("one");
    let two = Cell::new("two");
    let three = Cell::new("three");
    let four = Cell::new("four");

    // Test normal insertion.
    let mut cache: LRUCache<usize, Cell<&str>> = LRUCache::new(2); // (_, _) (cache is empty)
    cache.insert(1, one);    // (1, _)
    cache.insert(2, two);    // (1, 2)
    cache.insert(3, three);  // (2, 3)

    assert!(cache.find(&1).is_none());  // (2, 3) (no change)
    assert!(cache.find(&3).is_some());  // (2, 3)
    assert!(cache.find(&2).is_some());  // (3, 2)

    // Test that LRU works (this insertion should replace 3, not 2).
    cache.insert(4, four); // (2, 4)

    assert!(cache.find(&1).is_none());  // (2, 4) (no change)
    assert!(cache.find(&2).is_some());  // (4, 2)
    assert!(cache.find(&3).is_none());  // (4, 2) (no change)
    assert!(cache.find(&4).is_some());  // (2, 4) (no change)

    // Test find_or_create.
    cache.find_or_create(&1, |_| { Cell::new("one") }); // (4, 1)

    assert!(cache.find(&1).is_some()); // (4, 1) (no change)
    assert!(cache.find(&2).is_none()); // (4, 1) (no change)
    assert!(cache.find(&3).is_none()); // (4, 1) (no change)
    assert!(cache.find(&4).is_some()); // (1, 4)
}
