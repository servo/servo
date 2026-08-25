// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutcfullyear
description: >
  Date.prototype.setUTCFullYear.length is 3.
info: |
  Date.prototype.setUTCFullYear ( year [ , month [ , date ] ] )

  The "length" property of this method is 3𝔽.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the "length" property of a built-in function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Date.prototype.setUTCFullYear, "length", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: true,
});
