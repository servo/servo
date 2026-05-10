// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: length boundaries from ToLength operation
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...

  7.1.15 ToLength ( argument )

  1. Let len be ? ToInteger(argument).
  2. If len ≤ +0, return +0.
  3. If len is +∞, return 2**53-1.
  4. Return min(len, 2**53-1).
features: [Array.prototype.includes]
---*/

var obj = {
  "0": "a",
  "1": "b",
  "9007199254740990": "c", // 2 ** 53 - 2
  "9007199254740991": "d", // 2 ** 53 - 1
  "9007199254740992": "e", // 2 ** 53
};

obj.length = -0;
assert.sameValue([].includes.call(obj, "a"), false, "-0");

obj.length = -1;
assert.sameValue([].includes.call(obj, "a"), false, "-1");

obj.length = -0.1;
assert.sameValue([].includes.call(obj, "a"), false, "-0.1");

obj.length = -Infinity;
assert.sameValue([].includes.call(obj, "a"), false, "-Infinity");

var fromIndex = 9007199254740990;

obj.length = 9007199254740991;
assert.sameValue(
  [].includes.call(obj, "c", fromIndex),
  true,
  "2**53-1, found value at 2**53-2"
);

obj.length = 9007199254740991;
assert.sameValue(
  [].includes.call(obj, "d", fromIndex),
  false,
  "2**53-1, ignores indexes >= 2**53-1"
);

obj.length = 9007199254740992;
assert.sameValue(
  [].includes.call(obj, "d", fromIndex),
  false,
  "2**53, ignores indexes >= 2**53-1"
);

obj.length = 9007199254740993;
assert.sameValue(
  [].includes.call(obj, "d", fromIndex),
  false,
  "2**53+1, ignores indexes >= 2**53-1"
);

obj.length = Infinity;
assert.sameValue(
  [].includes.call(obj, "c", fromIndex),
  true,
  "Infinity, found item"
);
assert.sameValue(
  [].includes.call(obj, "d", fromIndex),
  false,
  "Infinity, ignores indexes >= 2**53-1"
);
