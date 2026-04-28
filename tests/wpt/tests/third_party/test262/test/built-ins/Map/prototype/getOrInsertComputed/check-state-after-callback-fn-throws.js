// Copyright (C) 2025 Daniel Minor. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Check state after callback function throws
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...

  6. Let value be ? Call(callbackfn, key).
  ...
features: [upsert]
---*/
var map = new Map();
map.set(0, 'zero');
map.set(1, 'one');
map.set(2, 'two');

assert.throws(Error, function() {
  map.getOrInsertComputed(3, function() {
    throw new Error('throw in callback');
  })
});

// Check the values after throwing in callbackfn.
assert.sameValue(map.get(0), 'zero');
assert.sameValue(map.get(1), 'one');
assert.sameValue(map.get(2), 'two');
assert.sameValue(map.has(3), false)

assert.throws(Error, function() {
  map.getOrInsertComputed(3, function() {
    map.set(1, 'mutated');
    throw new Error('throw in callback');
  })
});

// Check the values after throwing in callbackfn, with mutation.
assert.sameValue(map.get(0), 'zero');
assert.sameValue(map.get(1), 'mutated',);
assert.sameValue(map.get(2), 'two');
assert.sameValue(map.has(3), false)

assert.throws(Error, function() {
  map.getOrInsertComputed(3, function() {
    map.set(3, 'mutated');
    throw new Error('throw in callback');
  })
});

// Check the values after throwing in callbackfn, with mutation.
assert.sameValue(map.get(0), 'zero');
assert.sameValue(map.get(1), 'mutated',);
assert.sameValue(map.get(2), 'two');
assert.sameValue(map.get(3), 'mutated')
