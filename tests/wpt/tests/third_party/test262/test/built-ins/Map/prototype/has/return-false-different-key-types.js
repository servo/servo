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

assert.sameValue(map.has('str'), false);
assert.sameValue(map.has(1),  false);
assert.sameValue(map.has(NaN), false);
assert.sameValue(map.has(true), false);
assert.sameValue(map.has(false), false);
assert.sameValue(map.has({}), false);
assert.sameValue(map.has([]), false);
assert.sameValue(map.has(Symbol()), false);
assert.sameValue(map.has(null), false);
assert.sameValue(map.has(undefined), false);
