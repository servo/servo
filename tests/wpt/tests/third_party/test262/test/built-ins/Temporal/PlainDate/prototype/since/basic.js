// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = Temporal.PlainDate.from({year: 1969, monthCode: "M07", day: 24 });
const date2 = Temporal.PlainDate.from({year: 1969, monthCode: "M10", day: 5 });
TemporalHelpers.assertDuration(date2.since(date, { largestUnit: "days" }), 0, 0, 0, /* days = */ 73, 0, 0, 0, 0, 0, 0, "same year");

const earlier = date;
const later = Temporal.PlainDate.from({year: 1996, monthCode: "M03", day: 3 });
var duration = later.since(earlier, { largestUnit: "days" });
TemporalHelpers.assertDuration(duration, 0, 0, 0, /* days = */ 9719, 0, 0, 0, 0, 0, 0, "different year");

// Years

const date19971201 = Temporal.PlainDate.from({year: 1997, monthCode: "M12", day: 1 });
const date20010618 = Temporal.PlainDate.from({year: 2001, monthCode: "M06", day: 18 });
duration = date19971201.since(date20010618, { largestUnit: "years" });
TemporalHelpers.assertDuration(duration, -3, -6, 0, -17, 0, 0, 0, 0, 0, 0, "3 years, 6 months, 17 days");

// Months
const date20001201 = Temporal.PlainDate.from({year: 2000, monthCode: "M12", day: 1 });
const date20010601 = Temporal.PlainDate.from({year: 2001, monthCode: "M06", day: 1 });
duration = date20001201.since(date20010601, { largestUnit: "months" });
TemporalHelpers.assertDuration(duration, 0, -6, 0, 0, 0, 0, 0, 0, 0, 0, "6 months");

// Weeks
const date20000101 = Temporal.PlainDate.from({year: 2000, monthCode: "M01", day: 1 });
const date20001007 = Temporal.PlainDate.from({year: 2000, monthCode: "M10", day: 7 });
duration = date20000101.since(date20001007, { largestUnit: "weeks" });
TemporalHelpers.assertDuration(duration, 0, 0, -40, 0, 0, 0, 0, 0, 0, 0, "40 weeks");

// Days
duration = date20000101.since(date20001007, { largestUnit: "days" });
TemporalHelpers.assertDuration(duration, 0, 0, 0, -280, 0, 0, 0, 0, 0, 0, "40 weeks");
