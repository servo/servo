// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.delete
description: >
  Return false if an Object key is not in the WeakMap.
info: |
  WeakMap.prototype.delete ( _key_ )
  6. Return *false*.
features: [WeakMap]
---*/

var map = new WeakMap();
var foo = {};
var bar = {};

map.set(foo, 42);

assert.sameValue(map.delete(bar), false);
