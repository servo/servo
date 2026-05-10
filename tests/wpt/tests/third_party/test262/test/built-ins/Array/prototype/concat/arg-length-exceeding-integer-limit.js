// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: >
  TypeError is thrown if "length" of result array exceeds 2^53 - 1.
info: |
  Array.prototype.concat ( ...arguments )

  [...]
  5. Repeat, while items is not empty
    [...]
    c. If spreadable is true, then
      [...]
      ii. Let len be ? LengthOfArrayLike(E).
      iii. If n + len > 2^53 - 1, throw a TypeError exception.
    [...]
features: [Symbol.isConcatSpreadable, Proxy]
---*/

var spreadableLengthOutOfRange = {};
spreadableLengthOutOfRange.length = Number.MAX_SAFE_INTEGER;
spreadableLengthOutOfRange[Symbol.isConcatSpreadable] = true;

assert.throws(TypeError, function() {
  [1].concat(spreadableLengthOutOfRange);
}, '[1].concat(spreadableLengthOutOfRange) throws a TypeError exception');

var proxyForArrayWithLengthOutOfRange = new Proxy([], {
  get: function(_target, key) {
    if (key === "length") {
      return Number.MAX_SAFE_INTEGER;
    }
  },
});

assert.throws(TypeError, function() {
  [].concat(1, proxyForArrayWithLengthOutOfRange);
}, '[].concat(1, proxyForArrayWithLengthOutOfRange) throws a TypeError exception');
