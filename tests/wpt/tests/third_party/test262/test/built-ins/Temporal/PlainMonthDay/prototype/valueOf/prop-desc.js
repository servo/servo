// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.valueof
description: The "valueOf" property of Temporal.PlainMonthDay.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainMonthDay.prototype.valueOf,
  "function",
  "`typeof PlainMonthDay.prototype.valueOf` is `function`"
);

verifyProperty(Temporal.PlainMonthDay.prototype, "valueOf", {
  writable: true,
  enumerable: false,
  configurable: true,
});
