// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Half rounding modes at the exact 0.5 boundary for all units
info: |
  Tests that all rounding modes correctly break ties at the exact 0.5 boundary
  in RoundRelativeDuration, for both odd and even integer parts (distinguishing
  halfEven from other modes).

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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    yearEarlier1.until(yearLater, { smallestUnit: "years", roundingMode: mode }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `1.5 years with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    yearEarlier1.until(yearLater, { smallestUnit: "years", roundingMode: mode }),
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `1.5 years with ${mode} rounds up to 2`
  );
}

const yearEarlier2 = Temporal.ZonedDateTime.from("2018-01-01T00:00+00:00[UTC]");

assert.sameValue(
  yearEarlier2.until(yearLater).total({ unit: "years", relativeTo: yearEarlier2 }),
  2.5,
  "2.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    yearEarlier2.until(yearLater, { smallestUnit: "years", roundingMode: mode }),
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `2.5 years with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    yearEarlier2.until(yearLater, { smallestUnit: "years", roundingMode: mode }),
    3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `2.5 years with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    monthEarlier1.until(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    `1.5 months with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    monthEarlier1.until(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    `1.5 months with ${mode} rounds up to 2`
  );
}

const monthEarlier2 = Temporal.ZonedDateTime.from("2018-12-01T00:00+00:00[UTC]");

assert.sameValue(
  monthEarlier2.until(monthLater).total({ unit: "months", relativeTo: monthEarlier2 }),
  2.5,
  "2.5-month duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    monthEarlier2.until(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    `2.5 months with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    monthEarlier2.until(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, 3, 0, 0, 0, 0, 0, 0, 0, 0,
    `2.5 months with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    weekStart.until(weekLater1, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    `1.5 weeks with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    weekStart.until(weekLater1, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
    `1.5 weeks with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  weekStart.until(weekLater2).total({ unit: "weeks", relativeTo: weekStart }),
  2.5,
  "2.5-week duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    weekStart.until(weekLater2, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
    `2.5 weeks with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    weekStart.until(weekLater2, { smallestUnit: "weeks", roundingMode: mode }),
    0, 0, 3, 0, 0, 0, 0, 0, 0, 0,
    `2.5 weeks with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    dayStart.until(dayLater1, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    `1.5 days with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    dayStart.until(dayLater1, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, 2, 0, 0, 0, 0, 0, 0,
    `1.5 days with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  dayStart.until(dayLater2).total({ unit: "days", relativeTo: dayStart }),
  2.5,
  "2.5-day duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    dayStart.until(dayLater2, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, 2, 0, 0, 0, 0, 0, 0,
    `2.5 days with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    dayStart.until(dayLater2, { smallestUnit: "days", roundingMode: mode }),
    0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
    `2.5 days with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    hourStart.until(hourLater1, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    `1.5 hours with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    hourStart.until(hourLater1, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
    `1.5 hours with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  hourStart.until(hourLater2).total({ unit: "hours", relativeTo: hourStart }),
  2.5,
  "2.5-hour duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    hourStart.until(hourLater2, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
    `2.5 hours with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    hourStart.until(hourLater2, { smallestUnit: "hours", roundingMode: mode }),
    0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    `2.5 hours with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    minStart.until(minLater1, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    `1.5 minutes with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    minStart.until(minLater1, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
    `1.5 minutes with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  minStart.until(minLater2).total({ unit: "minutes", relativeTo: minStart }),
  2.5,
  "2.5-minute duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    minStart.until(minLater2, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
    `2.5 minutes with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    minStart.until(minLater2, { smallestUnit: "minutes", roundingMode: mode }),
    0, 0, 0, 0, 0, 3, 0, 0, 0, 0,
    `2.5 minutes with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    secStart.until(secLater1, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
    `1.5 seconds with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    secStart.until(secLater1, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
    `1.5 seconds with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  secStart.until(secLater2).total({ unit: "seconds", relativeTo: secStart }),
  2.5,
  "2.5-second duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    secStart.until(secLater2, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
    `2.5 seconds with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    secStart.until(secLater2, { smallestUnit: "seconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
    `2.5 seconds with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    msStart.until(msLater1, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
    `1.5 milliseconds with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    msStart.until(msLater1, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 2, 0, 0,
    `1.5 milliseconds with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  msStart.until(msLater2).total({ unit: "milliseconds", relativeTo: msStart }),
  2.5,
  "2.5-millisecond duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    msStart.until(msLater2, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 2, 0, 0,
    `2.5 milliseconds with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    msStart.until(msLater2, { smallestUnit: "milliseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 3, 0, 0,
    `2.5 milliseconds with ${mode} rounds up to 3`
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

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    usStart.until(usLater1, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    `1.5 microseconds with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    usStart.until(usLater1, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, 2, 0,
    `1.5 microseconds with ${mode} rounds up to 2`
  );
}

assert.sameValue(
  usStart.until(usLater2).total({ unit: "microseconds", relativeTo: usStart }),
  2.5,
  "2.5-microsecond duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    usStart.until(usLater2, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, 2, 0,
    `2.5 microseconds with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    usStart.until(usLater2, { smallestUnit: "microseconds", roundingMode: mode }),
    0, 0, 0, 0, 0, 0, 0, 0, 3, 0,
    `2.5 microseconds with ${mode} rounds up to 3`
  );
}
