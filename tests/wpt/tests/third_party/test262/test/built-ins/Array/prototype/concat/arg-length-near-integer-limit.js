// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: >
  Elements are copied from a spreadable array-like object
  whose "length" property is near the integer limit.
info: |
  Array.prototype.concat ( ...arguments )

  [...]
  5. Repeat, while items is not empty
    [...]
    c. If spreadable is true, then
      [...]
      ii. Let len be ? LengthOfArrayLike(E).
      iii. If n + len > 2^53 - 1, throw a TypeError exception.
      iv. Repeat, while k < len
        [...]
        3. If exists is true, then
          a. Let subElement be ? Get(E, P).
    [...]
features: [Symbol.isConcatSpreadable]
---*/

var spreadableHasPoisonedIndex = {
  length: Number.MAX_SAFE_INTEGER,
  get 0() {
    throw new Test262Error();
  },
};
spreadableHasPoisonedIndex[Symbol.isConcatSpreadable] = true;

assert.throws(Test262Error, function() {
  [].concat(spreadableHasPoisonedIndex);
}, '[].concat(spreadableHasPoisonedIndex) throws a Test262Error exception');
