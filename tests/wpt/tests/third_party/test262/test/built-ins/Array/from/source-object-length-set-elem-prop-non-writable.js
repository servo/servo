// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.from
description: >
  Non-writable properties are overwritten by CreateDataProperty.
  (result object's "0" is non-writable, items is not iterable)
info: |
  Array.from ( items [ , mapfn [ , thisArg ] ] )

  [...]
  4. Let usingIterator be ? GetMethod(items, @@iterator).
  5. If usingIterator is not undefined, then
    [...]
  6. NOTE: items is not an Iterable so assume it is an array-like object.
  [...]
  12. Repeat, while k < len
    [...]
    e. Perform ? CreateDataPropertyOrThrow(A, Pk, mappedValue).
  [...]
includes: [propertyHelper.js]
---*/

var items = {
  "0": 2,
  length: 1,
};

var A = function(_length) {
  Object.defineProperty(this, "0", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: true,
  });
};

var res = Array.from.call(A, items);

verifyProperty(res, "0", {
  value: 2,
  writable: true,
  enumerable: true,
  configurable: true,
});
