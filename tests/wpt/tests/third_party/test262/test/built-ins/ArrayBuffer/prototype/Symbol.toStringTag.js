// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype-@@tostringtag
description: >
    `Symbol.toStringTag` property descriptor
info: |
    The initial value of the @@toStringTag property is the String value
    "ArrayBuffer".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Symbol.toStringTag]
---*/

assert.sameValue(ArrayBuffer.prototype[Symbol.toStringTag], 'ArrayBuffer');

verifyProperty(ArrayBuffer.prototype, Symbol.toStringTag, {
  writable: false,
  enumerable: false,
  configurable: true,
});
