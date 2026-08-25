// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.flat
description: >
  Non-writable properties are overwritten by CreateDataProperty.
  (result object's "0" is non-writable, source array gets flattened)
info: |
  Array.prototype.flat ( [ depth ] )

  [...]
  6. Perform ? FlattenIntoArray(A, O, sourceLen, 0, depthNum).

  FlattenIntoArray ( target, source, sourceLen, start, depth [ , mapperFunction, thisArg ] )

  [...]
  9. Repeat, while sourceIndex < sourceLen
    [...]
    c. If exists is true, then
      [...]
      v. If shouldFlatten is true, then
        [...]
        2. Set targetIndex to ? FlattenIntoArray(target, element, elementLen, targetIndex, depth - 1).
      [...]
features: [Symbol.species]
includes: [propertyHelper.js]
---*/

var A = function(_length) {
  Object.defineProperty(this, "0", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: true,
  });
};

var arr = [[2]];
arr.constructor = {};
arr.constructor[Symbol.species] = A;

var res = arr.flat(1);

verifyProperty(res, "0", {
  value: 2,
  writable: true,
  enumerable: true,
  configurable: true,
});
