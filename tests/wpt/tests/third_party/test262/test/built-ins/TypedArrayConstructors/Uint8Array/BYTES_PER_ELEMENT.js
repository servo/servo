// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.bytes_per_element
description: >
  The initial value of Uint8Array.BYTES_PER_ELEMENT is 1.
info: |
  The value of TypedArray.BYTES_PER_ELEMENT is the Number value of the
  Element Size value specified in Table 49 for TypedArray.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [TypedArray]
---*/

assert.sameValue(Uint8Array.BYTES_PER_ELEMENT, 1);

verifyNotEnumerable(Uint8Array, "BYTES_PER_ELEMENT");
verifyNotWritable(Uint8Array, "BYTES_PER_ELEMENT");
verifyNotConfigurable(Uint8Array, "BYTES_PER_ELEMENT");
