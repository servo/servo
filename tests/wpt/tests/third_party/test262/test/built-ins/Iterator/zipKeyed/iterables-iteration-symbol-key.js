// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Symbol properties are used during "iterables" iteration.
includes: [compareArray.js]
features: [joint-iteration]
---*/

var symbolA = Symbol('a');

var iterables = {
  [symbolA]: ['value for a'],
  b: ['value for b'],
};

var result = Array.from(Iterator.zipKeyed(iterables));

assert.sameValue(result.length, 1);
assert.compareArray(Reflect.ownKeys(result[0]), ["b", symbolA]);
assert.sameValue(result[0].b, "value for b");
assert.sameValue(result[0][symbolA], "value for a");
