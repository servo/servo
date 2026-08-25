// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.totemporalinstant
description: >
  Date.prototype.toTemporalInstant.name is "toTemporalInstant".
info: |
  Date.prototype.toTemporalInstant ( )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Date.prototype.toTemporalInstant, "name", {
  value: "toTemporalInstant",
  writable: false,
  enumerable: false,
  configurable: true
});
