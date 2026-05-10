// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Add duration with various units and calculate correctly
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";

const durationCases = {
  months: {
    duration: { months: 6 },
    result: {
      year: 2001,
      month: 6,
      monthCode: "M05",
      day: 1,
    },
    startDate: {
      year: 2000,
      month: 12,
      day: 1
    }
  },
  years: {
    duration: {
      years: 3,
      months: 6
    },
    result: {
      year: 2001,
      month: 6,
      monthCode: "M05",
      day: 18,
    },
    startDate: {
      year: 1997,
      monthCode: "M12",
      day: 1
    }
  }
};
for (var [unit, {duration, result, startDate}] of Object.entries(durationCases)) {
  duration = Temporal.Duration.from(duration);

  const start = Temporal.PlainYearMonth.from({
    ...startDate,
    calendar
  });

  const end = start.add(duration);
  const diff = start.until(end, { largestUnit: unit });
  TemporalHelpers.assertDurationsEqual(diff, duration, `${unit}`);
}
