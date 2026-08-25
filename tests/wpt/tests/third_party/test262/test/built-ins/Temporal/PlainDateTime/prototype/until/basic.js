// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = Temporal.PlainDateTime.from({year: 1969, monthCode: "M07", day: 24, hour: 12, minute: 34 });
const date2 = Temporal.PlainDateTime.from({year: 1969, monthCode: "M10", day: 5, hour: 12, minute: 34 });
TemporalHelpers.assertDuration(date2.until(date, { largestUnit: "days" }), 0, 0, 0, /* days = */ -73, 0, 0, 0, 0, 0, 0, "same year");

const earlier = date;
const later = Temporal.PlainDateTime.from({year: 1996, monthCode: "M03", day: 3, hour: 12, minute: 34 });
var duration = later.until(earlier, { largestUnit: "days" });
TemporalHelpers.assertDuration(duration, 0, 0, 0, /* days = */ -9719, 0, 0, 0, 0, 0, 0, "different year");

// Years

const date19971201 = Temporal.PlainDateTime.from({year: 1997, monthCode: "M12", day: 1, hour: 12, minute: 34 });
const date20010618 = Temporal.PlainDateTime.from({year: 2001, monthCode: "M06", day: 18, hour: 12, minute: 34 });
duration = date19971201.until(date20010618, { largestUnit: "years" });
TemporalHelpers.assertDuration(duration, 3, 6, 0, 17, 0, 0, 0, 0, 0, 0, "3 years, 6 months, 17 days");

// Months
const date20001201 = Temporal.PlainDateTime.from({year: 2000, monthCode: "M12", day: 1, hour: 12, minute: 34 });
const date20010601 = Temporal.PlainDateTime.from({year: 2001, monthCode: "M06", day: 1, hour: 12, minute: 34 });
duration = date20001201.until(date20010601, { largestUnit: "months" });
TemporalHelpers.assertDuration(duration, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, "6 months");

// Weeks
const date20000101 = Temporal.PlainDateTime.from({year: 2000, monthCode: "M01", day: 1, hour: 12, minute: 34 });
const date20001007 = Temporal.PlainDateTime.from({year: 2000, monthCode: "M10", day: 7, hour: 12, minute: 34 });
duration = date20000101.until(date20001007, { largestUnit: "weeks" });
TemporalHelpers.assertDuration(duration, 0, 0, 40, 0, 0, 0, 0, 0, 0, 0, "40 weeks");

// Days
duration = date20000101.until(date20001007, { largestUnit: "days" });
TemporalHelpers.assertDuration(duration, 0, 0, 0, 280, 0, 0, 0, 0, 0, 0, "40 weeks");
