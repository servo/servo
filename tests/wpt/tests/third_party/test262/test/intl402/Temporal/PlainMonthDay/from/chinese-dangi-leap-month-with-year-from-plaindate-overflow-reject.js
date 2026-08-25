// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plainmonthday.from
features: [Temporal, Intl.Era-monthcode]
description: Check that reference years are correct in situations where user provides a year
---*/

// All of these should coerce to the equivalent day in the non-leap version of the month
// The years in entries of `plainDateCandidates` are the ones yielded by 1. ICU4X's
// algorithm and 2. ICU4C's algorithm. When creating a PlainDate and `overflow`
// is not `"reject"` it should be possible to actually create one or the other of these sets of
// PlainDates, depending on which library is used.
// However, when attempting to create a PlainMonthDay from these PlainDates, should throw
// if overflow: "reject"

const plainDateCandidates = [
  // ICU4X years
  { year: 1651, monthCode: "M01L", day: 29, referenceYear: 1972 },
  { year: 1461, monthCode: "M01L", day: 30, referenceYear: 1970 },
  { year: 1765, monthCode: "M02L", day: 30, referenceYear: 1972 },
  { year: 1718, monthCode: "M08L", day: 30, referenceYear: 1971 },
  { year: -5738, monthCode: "M09L", day: 30, referenceYear: 1972 },
  { year: -4098, monthCode: "M10L", day: 30, referenceYear: 1972 },
  { year: -2173, monthCode: "M11L", day: 30, referenceYear: 1970 },
  { year: 1403, monthCode: "M12L", day: 29, referenceYear: 1972 },
  { year: -180, monthCode: "M12L", day: 30, referenceYear: 1972 },
  // ICU4C years
  { year: 1898, monthCode: "M01L", day: 29, referenceYear: 1972 },
  { year: 1898, monthCode: "M01L", day: 30, referenceYear: 1970 },
  { year: 1830, monthCode: "M02L", day: 30, referenceYear: 1972 },
  { year: 1718, monthCode: "M08L", day: 30, referenceYear: 1971 },
  { year: 1843, monthCode: "M09L", day: 30, referenceYear: 1972 },
  { year: 1737, monthCode: "M10L", day: 30, referenceYear: 1972 },
  { year: 1889, monthCode: "M11L", day: 30, referenceYear: 1970 },
  { year: 1879, monthCode: "M12L", day: 29, referenceYear: 1972 },
  { year: 1784, monthCode: "M12L", day: 30, referenceYear: 1972 },
];

const calendars = ["chinese", "dangi"];

for (const calendar of calendars){
  for (const {year, monthCode, day, referenceYear} of plainDateCandidates){
    const pd = Temporal.PlainDate.from({ calendar, year, monthCode, day });
    if (pd.monthCode === monthCode && pd.day === day){
      assert.throws(RangeError, () => {
        const pmd = Temporal.PlainMonthDay.from(pd, {overflow: "reject"})
      }, `${year}, ${monthCode}, ${day} should not be valid with reject overflow`);
    }
  }
}
