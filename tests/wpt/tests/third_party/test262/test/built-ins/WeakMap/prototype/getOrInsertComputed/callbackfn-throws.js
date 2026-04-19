// Copyright (C) 2024 Jonas Haukenes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getorinsertcomputed
description: |
  WeakMap.getOrInsertComputed throws when callbackfn throws return if abrubt completion Call(callbackfn, key)
info: |
  WeakMap.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  6. Let value be ? Call(callbackfn, undefined, key).
  ...
features: [upsert, WeakMap]
---*/
var map = new WeakMap();

var bar = {};

assert.throws(Error, function() {
  map.getOrInsertComputed(bar, function() {
    throw new Error('throw in callback');
  })
}, Error);

