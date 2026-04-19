// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: The "from" property of Temporal.PlainMonthDay
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainMonthDay.from,
  "function",
  "`typeof PlainMonthDay.from` is `function`"
);

verifyProperty(Temporal.PlainMonthDay, "from", {
  writable: true,
  enumerable: false,
  configurable: true,
});
