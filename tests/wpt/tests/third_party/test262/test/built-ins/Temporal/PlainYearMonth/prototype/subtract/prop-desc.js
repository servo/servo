// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: The "subtract" property of Temporal.PlainYearMonth.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainYearMonth.prototype.subtract,
  "function",
  "`typeof PlainYearMonth.prototype.subtract` is `function`"
);

verifyProperty(Temporal.PlainYearMonth.prototype, "subtract", {
  writable: true,
  enumerable: false,
  configurable: true,
});
