// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all month codes (M01-M12) in Islamic-umalqura calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test that all month codes M01-M12 are valid for the Islamic-umalqura calendar
// Unlike the tabular islamic-civil and islamic-tbla calendars, islamic-umalqura
// is an observational calendar where any month can have either 29 or 30 days
// depending on the year. Therefore, all months support up to 30 days for PlainMonthDay.

const calendar = "islamic-umalqura";

const allMonths = [
  ["M01", 1972],
  ["M02", 1970],
  ["M03", 1971],
  ["M04", 1972],
  ["M05", 1971],
  ["M06", 1972],
  ["M07", 1969],
  ["M08", 1972],
  ["M09", 1972],
  ["M10", 1970],
  ["M11", 1972],
  ["M12", 1971],
];

for (const [monthCode, referenceYear30] of allMonths) {
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`);

  const pmd30 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
  TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 30, `${monthCode}-30`, referenceYear30);

  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 31 },
    { overflow: "constrain" }
  );
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 30, `day 31 should be constrained to 30 for ${monthCode}`, referenceYear30);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}
