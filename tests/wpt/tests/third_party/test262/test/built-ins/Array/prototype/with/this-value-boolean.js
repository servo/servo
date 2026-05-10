// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with casts primitive receivers to objects
info: |
  Array.prototype.with ( index, value )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

Boolean.prototype.length = 2;
Boolean.prototype[0] = 0;
Boolean.prototype[1] = 1;

assert.compareArray(Array.prototype.with.call(true, 0, 2), [2, 1]);
assert.compareArray(Array.prototype.with.call(false, 0, 2), [2, 1]);

/* Add length and indexed properties to `Boolean.prototype` */
Boolean.prototype.length = 3;
delete Boolean.prototype[0];
delete Boolean.prototype[1];
assert.compareArray(Array.prototype.with.call(true, 0, 2), [2, undefined, undefined]);
assert.compareArray(Array.prototype.with.call(false, 0, 2), [2, undefined, undefined]);
delete Boolean.prototype.length;
Boolean.prototype[0] = "monkeys";
Boolean.prototype[2] = "bogus";
assert.throws(RangeError,
              () => Array.prototype.with.call(true, 0, 2),
              "Array.prototype.with on object with undefined length");
assert.throws(RangeError,
              () => Array.prototype.with.call(false, 0, 2),
              "Array.prototype.with on object with undefined length");
