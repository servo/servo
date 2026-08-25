// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Leap second is a valid ISO string for a calendar in a property bag
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2019, 6);

const calendar = "2016-12-31T23:59:60";

const arg = { year: 2019, monthCode: "M06", calendar };
const result = instance.until(arg);
TemporalHelpers.assertDuration(
  result,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for calendar"
);
