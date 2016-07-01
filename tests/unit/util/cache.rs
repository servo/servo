/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use util::cache::HashCache;

#[test]
fn test_hashcache() {
    let mut cache: HashCache<usize, Cell<&str>> = HashCache::new();

    cache.insert(1, Cell::new("one"));
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_none());

    cache.find_or_create(2, || { Cell::new("two") });
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_some());
}
