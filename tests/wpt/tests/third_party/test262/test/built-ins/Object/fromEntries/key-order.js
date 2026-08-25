// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Key enumeration order of result objects matches the order of entries in the iterable.
esid: sec-object.fromentries
includes: [compareArray.js]
features: [Object.fromEntries]
---*/

var entries = [
  ['z', 1],
  ['y', 2],
  ['x', 3],
  ['y', 4],
];

var result = Object.fromEntries(entries);
assert.sameValue(result.z, 1);
assert.sameValue(result.y, 4);
assert.sameValue(result.x, 3);
assert.compareArray(Object.getOwnPropertyNames(result), ['z', 'y', 'x']);
