// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Add duration with various units and calculate correctly
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";

const durationCases = {
  days: {
    duration: { days: 280 },
    result: {
      year: 2000,
      month: 10,
      monthCode: "M10",
      day: 16,
    },
    startDate: {
      year: 2000,
      month: 1,
      day: 1
    }
  },
  weeks: {
    duration: { weeks: 40 },
    result: {
      year: 2000,
      month: 10,
      monthCode: "M10",
      day: 16,
    },
    startDate: {
      year: 2000,
      month: 1,
      day: 1
    }
  },
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
      months: 6,
      days: 17
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

  const start = Temporal.PlainDate.from({
    ...startDate,
    calendar
  });

  const end = start.add(duration);
  const calculatedStart = end.subtract(duration);
  const expectedCalculatedStart = duration.years !== 0 && !end.monthCode.endsWith("L") ? start.subtract({ months: 1 }) : start;

  TemporalHelpers.assertPlainDate(calculatedStart, expectedCalculatedStart.year, expectedCalculatedStart.month, expectedCalculatedStart.monthCode, expectedCalculatedStart.day, `${unit}`, expectedCalculatedStart.era, expectedCalculatedStart.eraYear);
}
