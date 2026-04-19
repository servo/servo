// Copyright (C) 2025 Daniel Minor. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Check that callbackfn receives undefined for this and exactly one argument.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...

  6. Let value be ? Call(callbackfn, key).
  ...
features: [upsert]
flags: [onlyStrict]
---*/
var map = new Map();

map.getOrInsertComputed(1, function(...args) {
  assert.sameValue(this, undefined);
  assert.sameValue(args.length, 1);
  assert.sameValue(args[0], 1);
});
