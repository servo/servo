// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  undefined-valued properties are skipped in "iterables" iteration.
includes: [compareArray.js]
features: [joint-iteration]
---*/


var iterables = {
  a: undefined,
  b: ['value for b'],
};

var result = Array.from(Iterator.zipKeyed(iterables));

assert.sameValue(result.length, 1);
assert.compareArray(Reflect.ownKeys(result[0]), ["b"]);
assert.compareArray(Object.values(result[0]), ["value for b"]);
