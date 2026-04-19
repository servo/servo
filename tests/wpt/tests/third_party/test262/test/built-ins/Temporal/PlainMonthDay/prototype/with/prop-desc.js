// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: The "with" property of Temporal.PlainMonthDay.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainMonthDay.prototype.with,
  "function",
  "`typeof PlainMonthDay.prototype.with` is `function`"
);

verifyProperty(Temporal.PlainMonthDay.prototype, "with", {
  writable: true,
  enumerable: false,
  configurable: true,
});
