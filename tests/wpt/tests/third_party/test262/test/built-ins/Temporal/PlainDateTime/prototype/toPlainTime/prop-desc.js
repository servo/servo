// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.toplaintime
description: The "toPlainTime" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.toPlainTime,
  "function",
  "`typeof PlainDateTime.prototype.toPlainTime` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "toPlainTime", {
  writable: true,
  enumerable: false,
  configurable: true,
});
