// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipKeyed
description: >
  Accepts String objects as inputs.
includes: [compareArray.js]
features: [joint-iteration]
---*/

var result = Array.from(Iterator.zipKeyed({
  a: Object("abc"),
  b: Object("123"),
}));

assert.sameValue(result.length, 3);
result.forEach(function (object) {
  assert.compareArray(Object.keys(object), ["a", "b"]);
});
assert.compareArray(Object.values(result[0]), ["a", "1"]);
assert.compareArray(Object.values(result[1]), ["b", "2"]);
assert.compareArray(Object.values(result[2]), ["c", "3"]);
