// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all month codes (M01-M13) in Ethiopic calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test that all month codes M01-M13 are valid for the Ethiopic calendar
// The Ethiopic calendar has 12 months of 30 days each, plus a 13th month (Pagumen)
// of 5 or 6 days

const calendar = "ethiopic";

// M01-M12: Regular months with 30 days each
const regularMonthCodes = [
  "M01", "M02", "M03", "M04", "M05", "M06",
  "M07", "M08", "M09", "M10", "M11", "M12"
];

for (const monthCode of regularMonthCodes) {
  // Test creation with monthCode
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`, 1972);

  // Test with day 30 (all regular months have 30 days)
  const pmd30 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
  TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 30, `${monthCode}-30`, 1972);

  // Test overflow: constrain to 30
  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 31 },
    { overflow: "constrain" }
  );
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 30, `day 31 should be constrained to 30 for ${monthCode}`, 1972);

  // Test overflow: reject should throw for day 31
  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}

// M13: Short month (Pagumen) with 5 or 6 days

// Test M13 with day 6 (maximum, valid in leap years)
// Reference year 1971
const pmdM13Day6 = Temporal.PlainMonthDay.from({ calendar, monthCode: "M13", day: 6 });
TemporalHelpers.assertPlainMonthDay(pmdM13Day6, "M13", 6, "M13-06", 1971);

// Test M13 overflow: constrain to maximum
const constrained = Temporal.PlainMonthDay.from(
  { calendar, monthCode: "M13", day: 7 },
  { overflow: "constrain" }
);
TemporalHelpers.assertPlainMonthDay(constrained, "M13", 6, "day 7 should be constrained to 6 for M13", 1971);

// Test M13 overflow: reject should throw for day 7
assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M13", day: 7 }, { overflow: "reject" });
}, "M13 with day 7 should throw with reject overflow");
