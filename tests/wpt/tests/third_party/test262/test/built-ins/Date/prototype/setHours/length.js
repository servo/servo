// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.sethours
description: >
  Date.prototype.setHours.length is 4.
info: |
  Date.prototype.setHours ( hour [ , min [ , sec [ , ms ] ] ] )

  The "length" property of this method is 4𝔽.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the "length" property of a built-in function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Date.prototype.setHours, "length", {
  value: 4,
  writable: false,
  enumerable: false,
  configurable: true,
});
