// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Returns the value set before getOrInsert from the specified key on different types.
info: |
  Map.prototype.getOrInsert ( key , value )

  ...
  4. For each Record { [[Key]], [[Value]] } p of M.[[MapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  ...
features: [Symbol, upsert]
---*/
var map = new Map();

map.set('bar', 0);
assert.sameValue(map.get('bar'), map.getOrInsert('bar', 1));
assert.sameValue(0, map.getOrInsert('bar', 1));

map.set(1, 42);
assert.sameValue(map.get(1), map.getOrInsert(1, 43));
assert.sameValue(42, map.getOrInsert(1, 43));

map.set(NaN, 1);
assert.sameValue(map.get(NaN), map.getOrInsert(NaN, 2));
assert.sameValue(1, map.getOrInsert(NaN, 2));

var item = {};
map.set(item, 2);
assert.sameValue(map.get(item), map.getOrInsert(item, 3));
assert.sameValue(2, map.getOrInsert(item, 3));

item = [];
map.set(item, 3);
assert.sameValue(map.get(item), map.getOrInsert(item, 4));
assert.sameValue(3, map.getOrInsert(item, 4));

item = Symbol('item');
map.set(item, 4);
assert.sameValue(map.get(item), map.getOrInsert(item, 5));
assert.sameValue(4, map.getOrInsert(item, 5));

item = null;
map.set(item, 5);
assert.sameValue(map.get(item), map.getOrInsert(item, 6));
assert.sameValue(5, map.getOrInsert(item, 6));

item = undefined;
map.set(item, 6);
assert.sameValue(map.get(item), map.getOrInsert(item, 7));
assert.sameValue(6, map.getOrInsert(item, 7));

