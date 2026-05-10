// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: The "PlainMonthDay" property of Temporal
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainMonthDay,
  "function",
  "`typeof PlainMonthDay` is `function`"
);

verifyProperty(Temporal, "PlainMonthDay", {
  writable: true,
  enumerable: false,
  configurable: true,
});
