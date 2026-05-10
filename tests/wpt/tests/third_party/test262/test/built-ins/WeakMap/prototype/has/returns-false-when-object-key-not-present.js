// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: >
  Return false when an Object key is not present in the WeakMap entries.
info: |
  WeakMap.prototype.has ( _key_ )
  6. Return *false*.
features: [WeakMap]
---*/

var foo = {};
var bar = {};
var map = new WeakMap();

assert.sameValue(map.has(foo), false);

map.set(foo, 1);
assert.sameValue(map.has(bar), false);

map.delete(foo);
assert.sameValue(map.has(foo), false);
