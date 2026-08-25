// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Test insertion of value returned from callback with different key types.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  7. Let p be the Record { [[Key]]: key, [[Value]]: value }.
  8. Append p to M.[[MapData]].
  9. Return p.[[Value]].
  ...
features: [Symbol, arrow-function, upsert]
---*/
var map = new Map();
var item = 'bar';
assert.sameValue(map.getOrInsertComputed(item, () => 0), 0);

item = 1;
assert.sameValue(map.getOrInsertComputed(item, () => 42), 42);

item = NaN;
assert.sameValue(map.getOrInsertComputed(item, () => 1), 1);

item = {};
assert.sameValue(map.getOrInsertComputed(item, () => 2), 2);

item = [];
assert.sameValue(map.getOrInsertComputed(item, () => 3), 3);

item = Symbol('item');
assert.sameValue(map.getOrInsertComputed(item, () => 4), 4);

item = null;
assert.sameValue(map.getOrInsertComputed(item, () => 5), 5);

item = undefined;
assert.sameValue(map.getOrInsertComputed(item, () => 6), 6);

