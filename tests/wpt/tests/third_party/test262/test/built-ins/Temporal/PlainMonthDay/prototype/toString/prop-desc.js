// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tostring
description: The "toString" property of Temporal.PlainMonthDay.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainMonthDay.prototype.toString,
  "function",
  "`typeof PlainMonthDay.prototype.toString` is `function`"
);

verifyProperty(Temporal.PlainMonthDay.prototype, "toString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
