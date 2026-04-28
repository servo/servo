// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplainyearmonth
description: The "toPlainYearMonth" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.toPlainYearMonth,
  "function",
  "`typeof PlainDate.prototype.toPlainYearMonth` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "toPlainYearMonth", {
  writable: true,
  enumerable: false,
  configurable: true,
});
