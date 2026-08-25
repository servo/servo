// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Ensure the canonical key is passed to the callback function.
info: |
  Map.prototype.getOrInsertComputed ( key, callbackfn )

  ...
  4. Set key to CanonicalizeKeyedCollectionKey(key).
  ...
  6. Let value be ? Call(callbackfn, key).
  ...

  CanonicalizeKeyedCollectionKey ( key )

  1. If key is -0𝔽, return +0𝔽.
  2. Return key.
features: [upsert]
---*/

for (var key of [-0, +0]) {
  var map = new Map();

  var canonicalKey;
  map.getOrInsertComputed(key, function(keyArg) {
    canonicalKey = keyArg;
  });

  assert.sameValue(+0, canonicalKey);
}
