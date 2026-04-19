// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.set
description: Returns `this` when new value is duplicate.
info: |
  WeakMap.prototype.set ( key, value )

  1. Let M be the this value.
  ...
  6. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. If p.[[key]] is not empty and SameValue(p.[[key]], key) is true, then
      i. Set p.[[value]] to value.
      ii. Return M.
  ...
---*/

var foo = {};
var map = new WeakMap([
  [foo, 1]
]);

assert.sameValue(map.set(foo, 1), map, '`map.set(foo, 1)` returns `map`');
