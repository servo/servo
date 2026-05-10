// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.from
description: >
  Non-writable properties are overwritten by CreateDataProperty.
  (result object's "0" is non-writable, items is iterable)
info: |
  Array.from ( items [ , mapfn [ , thisArg ] ] )

  [...]
  5. If usingIterator is not undefined, then
    [...]
    e. Repeat,
      [...]
      viii. Let defineStatus be CreateDataPropertyOrThrow(A, Pk, mappedValue).
    [...]
features: [generators]
includes: [propertyHelper.js]
---*/

var items = function* () {
  yield 2;
};

var A = function(_length) {
  Object.defineProperty(this, "0", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: true,
  });
};

var res = Array.from.call(A, items());

verifyProperty(res, "0", {
  value: 2,
  writable: true,
  enumerable: true,
  configurable: true,
});
