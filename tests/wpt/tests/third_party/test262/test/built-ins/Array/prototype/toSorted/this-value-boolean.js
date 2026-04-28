// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted converts booleans to objects
info: |
  Array.prototype.toSorted ( compareFn )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.toSorted.call(true), []);
assert.compareArray(Array.prototype.toSorted.call(false), []);


/* Add length and indexed properties to `Boolean.prototype` */
Boolean.prototype.length = 3;
assert.compareArray(Array.prototype.toSorted.call(true), [undefined, undefined, undefined]);
assert.compareArray(Array.prototype.toSorted.call(false), [undefined, undefined, undefined]);
delete Boolean.prototype.length;
Boolean.prototype[0] = "monkeys";
Boolean.prototype[2] = "bogus";
assert.compareArray(Array.prototype.toSorted.call(true), []);
assert.compareArray(Array.prototype.toSorted.call(false), []);
