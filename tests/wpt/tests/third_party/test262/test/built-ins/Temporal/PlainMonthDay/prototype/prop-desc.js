// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype
description: The "prototype" property of Temporal.PlainMonthDay
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.PlainMonthDay.prototype, "object");
assert.notSameValue(Temporal.PlainMonthDay.prototype, null);

verifyProperty(Temporal.PlainMonthDay, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
