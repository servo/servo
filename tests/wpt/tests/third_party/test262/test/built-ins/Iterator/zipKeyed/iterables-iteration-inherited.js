// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Inherited properties are skipped in "iterables" iteration.
includes: [compareArray.js]
features: [joint-iteration]
---*/

var parent = {
  get a() {
    throw new Test262Error("inherited properties should not be examined");
  },
}

var iterables = {
  __proto__: parent,
  b: ['value for b'],
};

var result = Array.from(Iterator.zipKeyed(iterables));

assert.sameValue(result.length, 1);
assert.compareArray(Object.keys(result[0]), ["b"]);
assert.compareArray(Object.values(result[0]), ["value for b"]);
