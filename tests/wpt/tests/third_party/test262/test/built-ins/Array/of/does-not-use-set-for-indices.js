// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.of
description: >
  Non-writable properties are overwritten by CreateDataProperty.
  (result object's "0" is non-writable)
info: |
  Array.of ( ...items )

  [...]
  7. Repeat, while k < len
    [...]
    c. Perform ? CreateDataPropertyOrThrow(A, Pk, kValue).
  [...]
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

var res = Array.of.call(A, 2);

verifyProperty(res, "0", {
  value: 2,
  writable: true,
  enumerable: true,
  configurable: true,
});
