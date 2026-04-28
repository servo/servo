// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: >
  Non-writable properties are overwritten by CreateDataProperty.
  (result object's "0" is non-writable, argument is not spreadable)
info: |
  Array.prototype.concat ( ...arguments )

  [...]
  5. Repeat, while items is not empty
    [...]
    c. If spreadable is true, then
      [...]
      iv. Repeat, while k < len
        [...]
        3. If exists is true, then
          [...]
          b. Perform ? CreateDataPropertyOrThrow(A, ! ToString(n), subElement).
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

var arr = [];
arr.constructor = {};
arr.constructor[Symbol.species] = A;

var res = arr.concat(2);

verifyProperty(res, "0", {
  value: 2,
  writable: true,
  enumerable: true,
  configurable: true,
});
