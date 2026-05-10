// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Returns the value set before getOrInsertComputed from the specified key on different types.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  5. For each Record { [[Key]], [[Value]] } p of M.[[MapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  ...
features: [Symbol, arrow-function, upsert]
---*/
var map = new Map();

var item = 'bar';
map.set(item, 0);
assert.sameValue(map.getOrInsertComputed(item, () => 1), 0);

item = 1;
map.set(item, 42);
assert.sameValue(map.getOrInsertComputed(item, () => 43), 42);

item = NaN;
map.set(item, 1);
assert.sameValue(map.getOrInsertComputed(item, () => 2), 1);

item = {};
map.set(item, 2);
assert.sameValue(map.getOrInsertComputed(item, () => 3), 2);

item = [];
map.set(item, 3);
assert.sameValue(map.getOrInsertComputed(item, () => 4), 3);

item = Symbol('item');
map.set(item, 4);
assert.sameValue(map.getOrInsertComputed(item, () => 5), 4);

item = null;
map.set(item, 5);
assert.sameValue(map.getOrInsertComputed(item, () => 6), 5);

item = undefined;
map.set(item, 6);
assert.sameValue(map.getOrInsertComputed(item, () => 7), 6);

