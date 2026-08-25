// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplainmonthday
description: The "toPlainMonthDay" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.toPlainMonthDay,
  "function",
  "`typeof PlainDate.prototype.toPlainMonthDay` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "toPlainMonthDay", {
  writable: true,
  enumerable: false,
  configurable: true,
});
