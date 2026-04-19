// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: The "equals" property of Temporal.PlainYearMonth.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainYearMonth.prototype.equals,
  "function",
  "`typeof PlainYearMonth.prototype.equals` is `function`"
);

verifyProperty(Temporal.PlainYearMonth.prototype, "equals", {
  writable: true,
  enumerable: false,
  configurable: true,
});
