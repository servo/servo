// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: The "toString" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.toString,
  "function",
  "`typeof PlainDateTime.prototype.toString` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "toString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
