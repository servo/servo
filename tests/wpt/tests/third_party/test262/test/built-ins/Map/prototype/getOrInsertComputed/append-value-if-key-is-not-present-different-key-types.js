// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Inserts the value for the specified key on different types, when key not present.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  7. Let p be the Record { [[Key]]: key, [[Value]]: value }.
  8. Append p to M.[[MapData]].
  ...
features: [Symbol, arrow-function, upsert]
---*/
var map = new Map();

var item = 'bar';
map.getOrInsertComputed(item, () => 0);
assert.sameValue(map.get(item), 0);

item = 1;
map.getOrInsertComputed(item, () => 42);
assert.sameValue(map.get(item), 42);

item = NaN;
map.getOrInsertComputed(item, () => 1);
assert.sameValue(map.get(item), 1);

item = {};
map.getOrInsertComputed(item, () => 2);
assert.sameValue(map.get(item), 2);

item = [];
map.getOrInsertComputed(item, () => 3);
assert.sameValue(map.get(item), 3);

item = Symbol('item');
map.getOrInsertComputed(item, () => 4);
assert.sameValue(map.get(item), 4);

item = null;
map.getOrInsertComputed(item, () => 5);
assert.sameValue(map.get(item), 5);

item = undefined;
map.getOrInsertComputed(item, () => 6);
assert.sameValue(map.get(item), 6);

