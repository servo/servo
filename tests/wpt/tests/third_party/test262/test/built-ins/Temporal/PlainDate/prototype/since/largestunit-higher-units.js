// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Tests calculations with higher largestUnit than the default of 'days'
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(1969, 7, 24);
const later = Temporal.PlainDate.from({ year: 2019, month: 7, day: 24 });
const duration = later.since(date, { largestUnit: "years" });
TemporalHelpers.assertDuration(duration, /* years = */ 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, "crossing epoch");

const feb20 = Temporal.PlainDate.from("2020-02-01");
const feb21 = Temporal.PlainDate.from("2021-02-01");
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "years" }), /* years = */ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, "start of February, years");
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "months" }), 0, /* months = */ 12, 0, 0, 0, 0, 0, 0, 0, 0, "start of February, months");
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "weeks" }), 0, 0, /* weeks = */ 52, /* days = */ 2, 0, 0, 0, 0, 0, 0, "start of February, weeks");

const lastFeb20 = Temporal.PlainDate.from("2020-02-29");
const lastFeb21 = Temporal.PlainDate.from("2021-02-28");
TemporalHelpers.assertDuration(lastFeb21.since(lastFeb20, { largestUnit: "years" }), 0, /* months = */ 11, 0, /* days = */ 28, 0, 0, 0, 0, 0, 0, "end of February, years");
TemporalHelpers.assertDuration(lastFeb21.since(lastFeb20, { largestUnit: "months" }), 0, /* months = */ 11, 0, /* days = */ 28, 0, 0, 0, 0, 0, 0, "end of February, months");
TemporalHelpers.assertDuration(lastFeb21.since(lastFeb20, { largestUnit: "weeks" }), 0, 0, /* weeks = */ 52, /* days = */ 1, 0, 0, 0, 0, 0, 0, "end of February, weeks");
