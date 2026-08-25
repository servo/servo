// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.bytes_per_element
description: BigInt64Array.BYTES_PER_ELEMENT property descriptor
info: |
  22.2.5.1 TypedArray.BYTES_PER_ELEMENT

  The value of TypedArray.BYTES_PER_ELEMENT is the Number value of the
  Element Size value specified in Table 52 for TypedArray.

  This property has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt64Array, "BYTES_PER_ELEMENT", {
  value: 8,
  writable: false,
  enumerable: false,
  configurable: false
});
