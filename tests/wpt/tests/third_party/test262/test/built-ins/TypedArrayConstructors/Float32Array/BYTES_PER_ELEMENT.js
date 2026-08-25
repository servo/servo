// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.bytes_per_element
description: >
  The initial value of Float32Array.BYTES_PER_ELEMENT is 4.
info: |
  The value of TypedArray.BYTES_PER_ELEMENT is the Number value of the
  Element Size value specified in Table 49 for TypedArray.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [TypedArray]
---*/

assert.sameValue(Float32Array.BYTES_PER_ELEMENT, 4);

verifyNotEnumerable(Float32Array, "BYTES_PER_ELEMENT");
verifyNotWritable(Float32Array, "BYTES_PER_ELEMENT");
verifyNotConfigurable(Float32Array, "BYTES_PER_ELEMENT");
