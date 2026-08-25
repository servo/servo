// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: The "withCalendar" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.withCalendar,
  "function",
  "`typeof PlainDate.prototype.withCalendar` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "withCalendar", {
  writable: true,
  enumerable: false,
  configurable: true,
});
