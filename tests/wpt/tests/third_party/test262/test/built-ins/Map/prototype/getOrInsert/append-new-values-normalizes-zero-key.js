// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Append a new value in the map normalizing +0 and -0.
info: |
  Map.prototype.getOrInsert ( key , value )

  ...
  3. Set key to CanonicalizeKeyedCollectionKey(key).
  4. For each Record { [[Key]], [[Value]] } p of M.[[MapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  5. Let p be the Record { [[Key]]: key, [[Value]]: value }.
  6. Append p to M.[[MapData]].
  ...
features: [Symbol, upsert]
---*/
var map = new Map();
map.getOrInsert(-0, 42);
assert.sameValue(map.get(0), 42);

map = new Map();
map.getOrInsert(+0, 43);
assert.sameValue(map.get(0), 43);
assert.sameValue(map.get(-0), 43);
