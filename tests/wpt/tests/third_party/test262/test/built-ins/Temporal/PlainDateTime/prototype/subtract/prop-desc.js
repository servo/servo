// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: The "subtract" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.subtract,
  "function",
  "`typeof PlainDateTime.prototype.subtract` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "subtract", {
  writable: true,
  enumerable: false,
  configurable: true,
});
