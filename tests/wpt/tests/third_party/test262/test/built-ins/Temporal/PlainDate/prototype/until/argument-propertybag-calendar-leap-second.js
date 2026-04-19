// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Leap second is a valid ISO string for a calendar in a property bag
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(1976, 11, 18);

const calendar = "2016-12-31T23:59:60";

const arg = { year: 1976, monthCode: "M11", day: 18, calendar };
const result = instance.until(arg);
TemporalHelpers.assertDuration(
  result,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for calendar"
);
