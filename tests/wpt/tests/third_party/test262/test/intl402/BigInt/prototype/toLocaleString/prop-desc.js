// Copyright (C) 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tolocalestring
description: Checks the "toLocaleString" property of the BigInt prototype object.
info: |
  BigInt.prototype.toLocaleString ( [ locales [ , options ] ] )

  17 ECMAScript Standard Built-in Objects:

    Every other data property described in clauses 18 through 26 and in
    Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.

includes: [propertyHelper.js]
features: [BigInt]
---*/

assert.sameValue(
  typeof BigInt.prototype.toLocaleString,
  "function",
  "typeof BigInt.prototype.toLocaleString is function"
);

verifyProperty(BigInt.prototype, "toLocaleString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
