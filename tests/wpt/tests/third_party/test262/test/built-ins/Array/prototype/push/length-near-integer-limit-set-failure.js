// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.push
description: >
  A value is inserted in an array-like object whose length property is near the integer limit.
  Unsuccessful [[Set]] raises a TypeError.
info: |
  Array.prototype.push ( ...items )

  [...]
  2. Let len be ? LengthOfArrayLike(O).
  [...]
  4. Let argCount be the number of elements in items.
  [...]
  6. Repeat, while items is not empty,
    a. Remove the first element from items and let E be the value of the element.
    b. Perform ? Set(O, ! ToString(len), E, true).
    c. Set len to len + 1.
  [...]

  OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

  [...]
  3. If IsDataDescriptor(ownDesc) is true, then
    a. If ownDesc.[[Writable]] is false, return false.
---*/

var arrayLike = {
  length: Number.MAX_SAFE_INTEGER - 3,
};

Object.defineProperty(arrayLike, Number.MAX_SAFE_INTEGER - 1, {
  value: 33,
  writable: false,
  enumerable: true,
  configurable: true,
});

assert.throws(TypeError, function() {
  Array.prototype.push.call(arrayLike, 1, 2, 3);
});

assert.sameValue(arrayLike[Number.MAX_SAFE_INTEGER - 3], 1);
assert.sameValue(arrayLike[Number.MAX_SAFE_INTEGER - 2], 2);
assert.sameValue(arrayLike[Number.MAX_SAFE_INTEGER - 1], 33);
