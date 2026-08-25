// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: The "withPlainTime" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.withPlainTime,
  "function",
  "`typeof PlainDateTime.prototype.withPlainTime` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "withPlainTime", {
  writable: true,
  enumerable: false,
  configurable: true,
});
