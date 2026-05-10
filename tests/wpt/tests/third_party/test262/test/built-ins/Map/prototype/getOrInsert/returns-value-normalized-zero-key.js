// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Returns the value from the specified key normalizing +0 and -0.
info: |
  Map.prototype.getOrInsert ( key , value )

  5. Set key to CanonicalizeKeyedCollectionKey(key).
  4. For each Record { [[Key]], [[Value]] } p of M.[[MapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  ...
features: [upsert]
---*/
var map = new Map();

map.set(+0, 42);
assert.sameValue(map.getOrInsert(-0, 1), 42);

map = new Map();
map.set(-0, 43);
assert.sameValue(map.getOrInsert(+0, 1), 43);

