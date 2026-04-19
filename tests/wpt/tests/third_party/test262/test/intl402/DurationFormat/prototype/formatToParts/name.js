// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: Checks the "name" property of Intl.DurationFormat.prototype.formatToParts().
info: |
  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [Intl.DurationFormat]
---*/

verifyProperty(Intl.DurationFormat.prototype.formatToParts, "name", {
  value: "formatToParts",
  writable: false,
  enumerable: false,
  configurable: true
});
