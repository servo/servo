// Copyright (C) 2025 Daniel Minor. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getorinsertcomputed
description: |
  Check state after callback function throws
info: |
  WeakMap.prototype.getOrInsertComputed ( key , callbackfn )

  ...

  6. Let value be ? Call(callbackfn, key).
  ...
features: [upsert, WeakMap, Symbol, symbols-as-weakmap-keys]
---*/
var map = new WeakMap();
const obj0 = {};
const obj1 = {};
const obj2 = {};
const obj3 = {};
map.set(obj0, 'zero');
map.set(obj1, 'one');
map.set(obj2, 'two');

assert.throws(Error, function() {
  map.getOrInsertComputed(Symbol('3'), function() {
    throw new Error('throw in callback');
  })
});

// Check the values after throwing in callbackfn.
assert.sameValue(map.get(obj0), 'zero');
assert.sameValue(map.get(obj1), 'one');
assert.sameValue(map.get(obj2), 'two');
assert.sameValue(map.has(obj3), false);

assert.throws(Error, function() {
  map.getOrInsertComputed(obj3, function() {
    map.set(obj1, 'mutated');
    throw new Error('throw in callback');
  })
});

// Check the values after throwing in callbackfn, with mutation.
assert.sameValue(map.get(obj0), 'zero');
assert.sameValue(map.get(obj1), 'mutated');
assert.sameValue(map.get(obj2), 'two');
assert.sameValue(map.has(obj3), false);

assert.throws(Error, function() {
  map.getOrInsertComputed(obj3, function() {
    map.set(obj3, 'mutated');
    throw new Error('throw in callback');
  })
});

// Check the values after throwing in callbackfn, with mutation.
assert.sameValue(map.get(obj0), 'zero');
assert.sameValue(map.get(obj1), 'mutated');
assert.sameValue(map.get(obj2), 'two');
assert.sameValue(map.get(obj3), 'mutated');
