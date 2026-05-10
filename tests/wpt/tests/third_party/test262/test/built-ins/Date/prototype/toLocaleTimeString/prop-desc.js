// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.tolocaletimestring
description: >
  Property descriptor for Date.prototype.toLocaleTimeString.
info: |
  Date.prototype.toLocaleTimeString ( [ reserved1 [ , reserved2 ] ] )

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 19 through 28 and in
    Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

verifyProperty(Date.prototype, "toLocaleTimeString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
