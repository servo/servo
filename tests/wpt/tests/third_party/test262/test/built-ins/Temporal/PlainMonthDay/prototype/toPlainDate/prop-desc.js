// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.toplaindate
description: The "toPlainDate" property of Temporal.PlainMonthDay.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainMonthDay.prototype.toPlainDate,
  "function",
  "`typeof PlainMonthDay.prototype.toPlainDate` is `function`"
);

verifyProperty(Temporal.PlainMonthDay.prototype, "toPlainDate", {
  writable: true,
  enumerable: false,
  configurable: true,
});
