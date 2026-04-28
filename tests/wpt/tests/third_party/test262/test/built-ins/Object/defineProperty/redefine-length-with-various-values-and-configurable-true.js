// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-exotic-objects-defineownproperty-p-desc
description: >
  Redefining "length" to `configurable: true` throws a TypeError exception
info: |
  ArraySetLength ( A, Desc )

  ValidateAndApplyPropertyDescriptor ( O, P, extensible, Desc, current )
---*/

let a = [1];

assert.throws(TypeError, () => {
  Object.defineProperty(a, "length", {
    configurable: true
  });
});
assert.throws(TypeError, () => {
  Object.defineProperty(a, "length", {
    value: 1,
    configurable: true
  });
});
assert.throws(TypeError, () => {
  Object.defineProperty(a, "length", {
    value: 2,
    configurable: true
  });
});
assert.throws(TypeError, () => {
  Object.defineProperty(a, "length", {
    value: 3,
    configurable: true
  });
});
