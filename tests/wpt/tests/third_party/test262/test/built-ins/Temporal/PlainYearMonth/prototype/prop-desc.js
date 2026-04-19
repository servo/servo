// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype
description: The "prototype" property of Temporal.PlainYearMonth
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.PlainYearMonth.prototype, "object");
assert.notSameValue(Temporal.PlainYearMonth.prototype, null);

verifyProperty(Temporal.PlainYearMonth, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
