// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: The "until" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.until,
  "function",
  "`typeof PlainDateTime.prototype.until` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "until", {
  writable: true,
  enumerable: false,
  configurable: true,
});
