// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.set
description: Returns `this` after setting a new value.
info: |
  WeakMap.prototype.set ( key, value )

  1. Let M be this value.
  ...
  9. Return M.

---*/

var map = new WeakMap();

assert.sameValue(map.set({}, 1), map, '`map.set({}, 1)` returns `map`');
