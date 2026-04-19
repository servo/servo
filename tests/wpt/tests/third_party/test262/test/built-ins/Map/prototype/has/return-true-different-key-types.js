// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.has
description: >
  Returns true for existing keys, using different key types.
info: |
  Map.prototype.has ( key )

  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    i. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true,
    return true.
  ...
features: [Symbol]
---*/

var map = new Map();

var obj = {};
var arr = [];
var symb = Symbol();

map.set('str', undefined);
map.set(1, undefined);
map.set(NaN, undefined);
map.set(true, undefined);
map.set(false, undefined);
map.set(obj, undefined);
map.set(arr, undefined);
map.set(symb, undefined);
map.set(null, undefined);
map.set(undefined, undefined);

assert.sameValue(map.has('str'), true);
assert.sameValue(map.has(1),  true);
assert.sameValue(map.has(NaN), true);
assert.sameValue(map.has(true), true);
assert.sameValue(map.has(false), true);
assert.sameValue(map.has(obj), true);
assert.sameValue(map.has(arr), true);
assert.sameValue(map.has(symb), true);
assert.sameValue(map.has(null), true);
assert.sameValue(map.has(undefined), true);
