// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with throws if the receiver is null or undefined
info: |
  %TypedArray%.prototype.with ( index, value )

  1. Let O be the this value.
  2. Perform ? ValidateTypedArray(O).
  ...
includes: [testTypedArray.js]
features: [TypedArray, change-array-by-copy]
---*/

var invalidValues = {
  'null': null,
  'undefined': undefined,
  'true': true,
  '"abc"': "abc",
  '12': 12,
  'Symbol()': Symbol(),
  '[1, 2, 3]': [1, 2, 3],
  '{ 0: 1, 1: 2, 2: 3, length: 3 }': { 0: 1, 1: 2, 2: 3, length: 3 },
  'Uint8Array.prototype': Uint8Array.prototype,
};

Object.keys(invalidValues).forEach(desc => {
  var value = invalidValues[desc];
  assert.throws(TypeError, () => {
    TypedArray.prototype.with.call(value, 0, 0);
  }, `${desc} is not a valid TypedArray`);
});
