// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plainmonthday.from
features: [Temporal, Intl.Era-monthcode]
description: Check that reference years are correct in situations where user provides a year
---*/

// these combinations of months and days have not occurred in the Chinese calendar
// since 1900 and will not occur before 2035
// All of these should throw
// Values for "year" other than 1234 are the out-of-range reference years that 1. ICU4X's
// algorithm and 2. ICU4C's algorithm produce. These are not to be used, because they
// do not agree and because they pre-date the adoption of the calendar in its current form.

const nonexistentMonthDays = [
  { year: 1234, monthCode: "M01L", day: 29, },
  { year: 1234, monthCode: "M01L", day: 30, },
  { year: 1234, monthCode: "M02L", day: 30, },
  { year: 1234, monthCode: "M08L", day: 30, },
  { year: 1234, monthCode: "M09L", day: 30, },
  { year: 1234, monthCode: "M10L", day: 30, },
  { year: 1234, monthCode: "M11L", day: 30, },
  { year: 1234, monthCode: "M12L", day: 29, },
  { year: 1234, monthCode: "M12L", day: 30, },
  // ICU4X years
  { year: 1651, monthCode: "M01L", day: 29, },
  { year: 1461, monthCode: "M01L", day: 30, },
  { year: 1765, monthCode: "M02L", day: 30, },
  { year: 1718, monthCode: "M08L", day: 30, },
  { year: -5738, monthCode: "M09L", day: 30, },
  { year: -4098, monthCode: "M10L", day: 30, },
  { year: -2173, monthCode: "M11L", day: 30, },
  { year: 1403, monthCode: "M12L", day: 29, },
  { year: -180, monthCode: "M12L", day: 30, },
  // ICU4C years
  { year: 1898, monthCode: "M01L", day: 29, },
  { year: 1898, monthCode: "M01L", day: 30, },
  { year: 1830, monthCode: "M02L", day: 30, },
  { year: 1718, monthCode: "M08L", day: 30, },
  { year: 1843, monthCode: "M09L", day: 30, },
  { year: 1737, monthCode: "M10L", day: 30, },
  { year: 1890, monthCode: "M11L", day: 30, },
  { year: 1879, monthCode: "M12L", day: 29, },
  { year: 1784, monthCode: "M12L", day: 30, },
];

const calendars = ["chinese", "dangi"];

for (let calendar of calendars){
  for (let {year, monthCode, day} of nonexistentMonthDays){
    assert.throws(RangeError, () => {
      const pmd = Temporal.PlainMonthDay.from({calendar, year, monthCode, day}, {overflow: "reject" })
    }, `${year}, ${monthCode}, ${day} should not be valid with reject overflow`);
  }
}
