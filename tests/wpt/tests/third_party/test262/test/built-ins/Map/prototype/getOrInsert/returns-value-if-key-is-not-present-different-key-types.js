// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Returns the value from the specified key on different types, when key not present.
info: |
  Map.prototype.getOrInsert ( key , value )

  ...
  5. Let p be the Record { [[Key]]: key, [[Value]]: value }.
  6. Append p to M.[[MapData]].
  7. Return p.[[Value]].
  ...
features: [Symbol, upsert]
---*/
var map = new Map();

assert.sameValue(map.getOrInsert('bar', 0), 0);

assert.sameValue(map.getOrInsert(1, 42), 42);

assert.sameValue(map.getOrInsert(NaN, 1), 1);

var item = {};
assert.sameValue(map.getOrInsert(item, 2), 2);

item = [];
assert.sameValue(map.getOrInsert(item, 3), 3);

item = Symbol('item');
assert.sameValue(map.getOrInsert(item, 4), 4);

item = null;
assert.sameValue(map.getOrInsert(item, 5), 5);

item = undefined;
assert.sameValue(map.getOrInsert(item, 6), 6);

