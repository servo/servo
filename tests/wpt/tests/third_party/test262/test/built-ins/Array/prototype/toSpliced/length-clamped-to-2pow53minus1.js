// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Length is clamped to 2^53-1 when they exceed the integer limit.
info: |
  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...

  ToLength ( argument )

  1. Let len be ? ToIntegerOrInfinity(argument).
  2. If len ≤ 0, return +0𝔽.
  3. Return 𝔽(min(len, 2^53 - 1))
features: [change-array-by-copy, exponentiation]
includes: [compareArray.js]
---*/

var arrayLike = {
  "9007199254740989": 2 ** 53 - 3,
  "9007199254740990": 2 ** 53 - 2,
  "9007199254740991": 2 ** 53 - 1,
  "9007199254740992": 2 ** 53,
  "9007199254740994": 2 ** 53 + 2, // NOTE: 2 ** 53 + 1 is 2 ** 53
  length: 2 ** 53 + 20,
};

var result = Array.prototype.toSpliced.call(arrayLike, 0, 2 ** 53 - 3);

assert.sameValue(result.length, 2);
assert.compareArray(result, [2 ** 53 - 3, 2 ** 53 - 2]);
