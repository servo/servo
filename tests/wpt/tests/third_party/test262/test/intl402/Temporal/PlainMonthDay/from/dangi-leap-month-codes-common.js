// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for common leap month codes in Dangi calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test common leap months in the Dangi calendar
// Dangi is a lunisolar calendar where leap months follow the same distribution as Chinese
// Common leap months (occurring more frequently in the astronomical cycle):
// M02L, M03L, M04L, M05L, M06L, M07L, M08L
//
// The distribution of leap months follows astronomical calculations.

// Reference year for days 1-29 of leap months
//
// Month -> ISO year
//
// M01L     -
// M02L     1947
// M03L     1966
// M04L     1963
// M05L     1971
// M06L     1960
// M07L     1968
// M08L     1957
// M09L     2014
// M10L     1984
// M11L     2033
// M12L     -
//
// M01L and M12L are not known to have occurred in the range in which the
// Dangi calendar can be accurately calculated. M09L, M10L, and M11L are
// uncommon and did not occur between 1900-1972, so require reference years in
// the more recent past or near future.

const monthCodesWithYears = [
  { monthCode: "M02L", referenceYear1: 1947 },
  { monthCode: "M03L", referenceYear1: 1966, has30: true },
  { monthCode: "M04L", referenceYear1: 1963, has30: true },
  { monthCode: "M05L", referenceYear1: 1971, has30: true },
  { monthCode: "M06L", referenceYear1: 1960, has30: true },
  { monthCode: "M07L", referenceYear1: 1968, has30: true },
  { monthCode: "M08L", referenceYear1: 1957 },
  { monthCode: "M09L", referenceYear1: 2014 },
  { monthCode: "M10L", referenceYear1: 1984 },
  { monthCode: "M11L", referenceYear1: 2033, referenceYear29: 2034 },
];

const calendar = "dangi";

for (const { monthCode, referenceYear1, referenceYear29 = referenceYear1, has30 = false } of monthCodesWithYears) {
  // Test creation with monthCode
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`, referenceYear1);

  // These leap months can have at least 29 days
  const pmd29 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 29 });
  TemporalHelpers.assertPlainMonthDay(pmd29, monthCode, 29, `${monthCode}-29`, referenceYear29);

  // Test constraining; leap months that never occurred with 30 days should
  // constrain to the regular month. See dangi-30-day-leap-months.js for
  // constrain behaviour in leap months with 30 days
  if (!has30) {
    const regularMonth = monthCode.slice(0, 3);

    // Test constrain overflow - day 30 should be constrained to day 30 of the
    // regular month
    const constrain30 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
    assert.sameValue(constrain30.monthCode, regularMonth, `${monthCode}-30 should be constrained to ${regularMonth}-30`);
    assert.sameValue(constrain30.day, 30, `day 30 should be preserved for ${monthCode}`);
    // Reference year is tested in dangi-month-codes.js

    // Test constrain overflow - day 31 should be constrained to day 30 of the
    // regular month
    const constrain31 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 });
    assert.sameValue(constrain31.monthCode, regularMonth, `${monthCode}-31 should be constrained to ${regularMonth}-30`);
    assert.sameValue(constrain31.day, 30, `day 31 should be constrained to 30 for ${monthCode}`);
    // Reference year is tested in dangi-month-codes.js

    // Test reject overflow - day 30 should throw
    assert.throws(RangeError, () => {
      Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 }, { overflow: "reject" });
    }, `${monthCode}-30 should throw with reject`);
  }

  // Test reject overflow - day 31 should throw
  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}
