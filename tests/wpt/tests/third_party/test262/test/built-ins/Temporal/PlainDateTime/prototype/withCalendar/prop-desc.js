// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withcalendar
description: The "withCalendar" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.withCalendar,
  "function",
  "`typeof PlainDateTime.prototype.withCalendar` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "withCalendar", {
  writable: true,
  enumerable: false,
  configurable: true,
});
