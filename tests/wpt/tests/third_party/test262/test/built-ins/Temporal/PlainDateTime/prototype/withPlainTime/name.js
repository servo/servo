// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: Temporal.PlainDateTime.prototype.withPlainTime.name is "withPlainTime".
info: |
    Every built-in function object, including constructors, that is not identified as an anonymous
    function has a "name" property whose value is a String. Unless otherwise specified, this value
    is the name that is given to the function in this specification.

    Unless otherwise specified, the "name" property of a built-in function object, if it exists,
    has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.PlainDateTime.prototype.withPlainTime, "name", {
  value: "withPlainTime",
  writable: false,
  enumerable: false,
  configurable: true,
});
