// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced limits the length to 2 ** 32 - 1
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  3. Let len be ? LengthOfArrayLike(O).
  ...
  11. Let newLen be len + insertCount - actualDeleteCount.
  12. If _newLen_ > 2 ** 53< - 1, throw a *TypeError* exception.
  13. Let A be ? ArrayCreate(𝔽(newLen)).
  ...

  ArrayCreate ( length [, proto ] )

  1. If length > 2 ** 32 - 1, throw a RangeError exception.
features: [change-array-by-copy, exponentiation]
---*/

// Object with large "length" property
var arrayLike = {
  get "0"() {
    throw new Test262Error("Get 0");
  },
  get "4294967295" () { // 2 ** 32 - 1
    throw new Test262Error("Get 4294967295");
  },
  get "4294967296" () { // 2 ** 32
    throw new Test262Error("Get 4294967296");
  },
  length: 2 ** 32
};

assert.throws(RangeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0);
});

arrayLike.length = 2 ** 32 - 1;
assert.throws(RangeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});

arrayLike.length = 2 ** 32;
assert.throws(RangeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});

arrayLike.length = 2 ** 32 + 1;
assert.throws(RangeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});

arrayLike.length = 2 ** 52 - 2;
assert.throws(RangeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});

arrayLike.length = 2 ** 53 - 1;
assert.throws(TypeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});

arrayLike.length = 2 ** 53;
assert.throws(TypeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});

arrayLike.length = 2 ** 53 + 1;
assert.throws(TypeError, function() {
  Array.prototype.toSpliced.call(arrayLike, 0, 0, 1);
});
