// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Calculation is performed if calendars' IDs match
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const plainDate1 = new Temporal.PlainDate(2000, 1, 1, "iso8601");
const plainDate2 = Temporal.PlainDate.from({ year: 2000, month: 1, day: 2, calendar: "2024-05-16[u-ca=iso8601]" });
TemporalHelpers.assertDuration(plainDate1.until(plainDate2), 0, 0, 0, /* days = */ 1, 0, 0, 0, 0, 0, 0);
