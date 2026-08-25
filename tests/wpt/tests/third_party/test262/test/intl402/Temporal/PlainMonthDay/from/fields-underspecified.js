// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Throw a RangeError if only one of era/eraYear fields is present
features: [Temporal]
---*/

const tests = [
  ["gregory", { year: 2000, month: 5, day: 2, era: "ce" }, "era present but not eraYear"],
  ["gregory", { year: 2000, month: 5, day: 2, eraYear: 1 }, "eraYear present but not era"],
  ["gregory", { month: 8, day: 1 }, "no monthCode or year specification, non-ISO Gregorian"],
  ["hebrew", { month: 8, day: 1 }, "no monthCode or year specification, non-ISO non-Gregorian"],
];

for (const [calendarId, arg, description] of tests) {
  assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ ...arg, calendar: calendarId }), description);
}
