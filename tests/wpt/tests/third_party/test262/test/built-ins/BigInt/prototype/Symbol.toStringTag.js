// Copyright (C) 2017 Igalia, S. L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bigint-@@tostringtag
description: >
    `Symbol.toStringTag` property descriptor
info: |
    The initial value of the @@toStringTag property is the String value
    "BigInt".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [BigInt, Symbol, Symbol.toStringTag]
---*/

verifyProperty(BigInt.prototype, Symbol.toStringTag, {
  value: "BigInt",
  writable: false,
  enumerable: false,
  configurable: true
});
