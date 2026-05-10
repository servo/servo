// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  "with" property of %TypedArray%.prototype
info: |
  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [testTypedArray.js, propertyHelper.js]
features: [TypedArray, change-array-by-copy]
---*/

assert.sameValue(typeof TypedArray.prototype.with, "function", "typeof");

verifyProperty(TypedArray.prototype, "with", {
  value: TypedArray.prototype.with,
  writable: true,
  enumerable: false,
  configurable: true,
});
