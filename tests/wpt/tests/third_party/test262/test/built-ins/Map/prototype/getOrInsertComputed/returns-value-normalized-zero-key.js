// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Returns the value from the specified key normalizing +0 and -0.
info: |
  Map.prototype.getOrInsertComputed ( key , Callbackfn )

  4. Set key to CanonicalizeKeyedCollectionKey(key).
  5. For each Record { [[Key]], [[Value]] } p of M.[[MapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  ...

features: [arrow-function, upsert]
---*/
var map = new Map();

map.set(+0, 42);
assert.sameValue(map.getOrInsertComputed(-0, () => 1), 42);

map = new Map();
map.set(-0, 43);
assert.sameValue(map.getOrInsertComputed(+0, () => 1), 43);

