// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Half rounding modes at the exact 0.5 boundary for all units
info: |
  Tests that all rounding modes correctly break ties at the exact 0.5 boundary
  in RoundRelativeDuration in the negative direction, for both odd and even
  integer parts (distinguishing halfEven from other modes). In the negative
  direction, halfExpand and halfCeil diverge (away from zero vs toward positive
  infinity).

  Years: 2019-01-01 / 2020-07-02 → 1 year + 183 days, 183/366 = 0.5.
  Months: 2019-01-01 / 2019-02-15 → 1 month + 14 days, 14/28 = 0.5.
  Weeks: 2019-01-01T00:00 / 2019-01-11T12:00 → 1 week + 3.5 days, 3.5/7 = 0.5.
  Days: 2019-01-01T00:00 / 2019-01-02T12:00 → 1 day + 12 hours, 12/24 = 0.5.
  Hours: 1 hour + 30 minutes, 30/60 = 0.5.
  Minutes: 1 minute + 30 seconds, 30/60 = 0.5.
  Seconds: 1 second + 500 ms, 500/1000 = 0.5.
  Milliseconds: 1 ms + 500 µs, 500/1000 = 0.5.
  Microseconds: 1 µs + 500 ns, 500/1000 = 0.5.

  Each unit is tested with an odd integer part (N.5) and an even integer part
  ((N+1).5) to distinguish halfEven from halfExpand.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// --- years ---

const yearEarlier1 = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const yearLater = Temporal.ZonedDateTime.from("2020-07-02T00:00+00:00[UTC]");

assert.sameValue(
  yearEarlier1.until(yearLater).total({ unit: "years", relativeTo: yearEarlier1 }),
  1.5,
  "1.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    yearEarlier1.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 years with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    yearEarlier1.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 years with ${mode} rounds away from zero`
  );
}

const yearEarlier2 = Temporal.ZonedDateTime.from("2018-01-01T00:00+00:00[UTC]");

assert.sameValue(
  yearEarlier2.until(yearLater).total({ unit: "years", relativeTo: yearEarlier2 }),
  2.5,
  "2.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    yearEarlier2.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 years with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    yearEarlier2.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 years with ${mode} rounds away from zero`
  );
}

// --- months ---

const monthEarlier1 = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const monthLater = Temporal.ZonedDateTime.from("2019-02-15T00:00+00:00[UTC]");

assert.sameValue(
  monthEarlier1.until(monthLater).total({ unit: "months", relativeTo: monthEarlier1 }),
  1.5,
  "1.5-month duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    monthEarlier1.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 months with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    monthEarlier1.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -2, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 months with ${mode} rounds away from zero`
  );
}

const monthEarlier2 = Temporal.ZonedDateTime.from("2018-12-01T00:00+00:00[UTC]");

assert.sameValue(
  monthEarlier2.until(monthLater).total({ unit: "months", relativeTo: monthEarlier2 }),
  2.5,
  "2.5-month duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    monthEarlier2.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -2, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 months with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    monthEarlier2.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -3, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 months with ${mode} rounds away from zero`
  );
}

// --- weeks ---

const weekStart = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const weekLater1 = Temporal.ZonedDateTime.from("2019-01-11T12:00+00:00[UTC]"); // 1.5 weeks
const weekLater2 = Temporal.ZonedDateTime.from("2019-01-18T12:00+00:00[UTC]"); // 2.5 weeks

assert.sameValue(
  weekStart.until(weekLater1).total({ unit: "weeks", relativeTo: weekStart }),
  1.5,
  "1.5-week duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    weekStart.since(weekLater1, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 weeks with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    weekStart.since(weekLater1, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, -2, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 weeks with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  weekStart.until(weekLater2).total({ unit: "weeks", relativeTo: weekStart }),
  2.5,
  "2.5-week duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    weekStart.since(weekLater2, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, -2, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 weeks with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    weekStart.since(weekLater2, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, -3, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 weeks with ${mode} rounds away from zero`
  );
}

// --- days ---

const dayStart = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const dayLater1 = Temporal.ZonedDateTime.from("2019-01-02T12:00+00:00[UTC]"); // 1.5 days
const dayLater2 = Temporal.ZonedDateTime.from("2019-01-03T12:00+00:00[UTC]"); // 2.5 days

assert.sameValue(
  dayStart.until(dayLater1).total({ unit: "days", relativeTo: dayStart }),
  1.5,
  "1.5-day duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    dayStart.since(dayLater1, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, -1, 0, 0, 0, 0, 0, 0,
    `-1.5 days with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    dayStart.since(dayLater1, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, -2, 0, 0, 0, 0, 0, 0,
    `-1.5 days with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  dayStart.until(dayLater2).total({ unit: "days", relativeTo: dayStart }),
  2.5,
  "2.5-day duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    dayStart.since(dayLater2, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, -2, 0, 0, 0, 0, 0, 0,
    `-2.5 days with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    dayStart.since(dayLater2, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
    `-2.5 days with ${mode} rounds away from zero`
  );
}

// --- hours ---

const hourStart = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const hourLater1 = Temporal.ZonedDateTime.from("2019-01-01T01:30+00:00[UTC]"); // 1.5 hours
const hourLater2 = Temporal.ZonedDateTime.from("2019-01-01T02:30+00:00[UTC]"); // 2.5 hours

assert.sameValue(
  hourStart.until(hourLater1).total({ unit: "hours", relativeTo: hourStart }),
  1.5,
  "1.5-hour duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    hourStart.since(hourLater1, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, -1, 0, 0, 0, 0, 0,
    `-1.5 hours with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    hourStart.since(hourLater1, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, -2, 0, 0, 0, 0, 0,
    `-1.5 hours with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  hourStart.until(hourLater2).total({ unit: "hours", relativeTo: hourStart }),
  2.5,
  "2.5-hour duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    hourStart.since(hourLater2, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, -2, 0, 0, 0, 0, 0,
    `-2.5 hours with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    hourStart.since(hourLater2, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, -3, 0, 0, 0, 0, 0,
    `-2.5 hours with ${mode} rounds away from zero`
  );
}

// --- minutes ---

const minStart = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const minLater1 = Temporal.ZonedDateTime.from("2019-01-01T00:01:30+00:00[UTC]"); // 1.5 minutes
const minLater2 = Temporal.ZonedDateTime.from("2019-01-01T00:02:30+00:00[UTC]"); // 2.5 minutes

assert.sameValue(
  minStart.until(minLater1).total({ unit: "minutes", relativeTo: minStart }),
  1.5,
  "1.5-minute duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    minStart.since(minLater1, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, -1, 0, 0, 0, 0,
    `-1.5 minutes with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    minStart.since(minLater1, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, -2, 0, 0, 0, 0,
    `-1.5 minutes with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  minStart.until(minLater2).total({ unit: "minutes", relativeTo: minStart }),
  2.5,
  "2.5-minute duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    minStart.since(minLater2, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, -2, 0, 0, 0, 0,
    `-2.5 minutes with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    minStart.since(minLater2, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, -3, 0, 0, 0, 0,
    `-2.5 minutes with ${mode} rounds away from zero`
  );
}

// --- seconds ---

const secStart = Temporal.ZonedDateTime.from("2019-01-01T00:00:00+00:00[UTC]");
const secLater1 = Temporal.ZonedDateTime.from("2019-01-01T00:00:01.5+00:00[UTC]");   // 1.5 seconds
const secLater2 = Temporal.ZonedDateTime.from("2019-01-01T00:00:02.5+00:00[UTC]");   // 2.5 seconds

assert.sameValue(
  secStart.until(secLater1).total({ unit: "seconds", relativeTo: secStart }),
  1.5,
  "1.5-second duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    secStart.since(secLater1, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, -1, 0, 0, 0,
    `-1.5 seconds with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    secStart.since(secLater1, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, -2, 0, 0, 0,
    `-1.5 seconds with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  secStart.until(secLater2).total({ unit: "seconds", relativeTo: secStart }),
  2.5,
  "2.5-second duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    secStart.since(secLater2, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, -2, 0, 0, 0,
    `-2.5 seconds with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    secStart.since(secLater2, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, -3, 0, 0, 0,
    `-2.5 seconds with ${mode} rounds away from zero`
  );
}

// --- milliseconds ---

const msStart = Temporal.ZonedDateTime.from("2019-01-01T00:00:00+00:00[UTC]");
const msLater1 = Temporal.ZonedDateTime.from("2019-01-01T00:00:00.0015+00:00[UTC]");  // 1.5 ms
const msLater2 = Temporal.ZonedDateTime.from("2019-01-01T00:00:00.0025+00:00[UTC]");  // 2.5 ms

assert.sameValue(
  msStart.until(msLater1).total({ unit: "milliseconds", relativeTo: msStart }),
  1.5,
  "1.5-millisecond duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    msStart.since(msLater1, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, -1, 0, 0,
    `-1.5 milliseconds with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    msStart.since(msLater1, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, -2, 0, 0,
    `-1.5 milliseconds with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  msStart.until(msLater2).total({ unit: "milliseconds", relativeTo: msStart }),
  2.5,
  "2.5-millisecond duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    msStart.since(msLater2, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, -2, 0, 0,
    `-2.5 milliseconds with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    msStart.since(msLater2, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, -3, 0, 0,
    `-2.5 milliseconds with ${mode} rounds away from zero`
  );
}

// --- microseconds ---

const usStart = Temporal.ZonedDateTime.from("2019-01-01T00:00:00+00:00[UTC]");
const usLater1 = Temporal.ZonedDateTime.from("2019-01-01T00:00:00.0000015+00:00[UTC]"); // 1.5 µs
const usLater2 = Temporal.ZonedDateTime.from("2019-01-01T00:00:00.0000025+00:00[UTC]"); // 2.5 µs

assert.sameValue(
  usStart.until(usLater1).total({ unit: "microseconds", relativeTo: usStart }),
  1.5,
  "1.5-microsecond duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    usStart.since(usLater1, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, -1, 0,
    `-1.5 microseconds with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    usStart.since(usLater1, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, -2, 0,
    `-1.5 microseconds with ${mode} rounds away from zero`
  );
}

assert.sameValue(
  usStart.until(usLater2).total({ unit: "microseconds", relativeTo: usStart }),
  2.5,
  "2.5-microsecond duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    usStart.since(usLater2, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, -2, 0,
    `-2.5 microseconds with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    usStart.since(usLater2, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, -3, 0,
    `-2.5 microseconds with ${mode} rounds away from zero`
  );
}
