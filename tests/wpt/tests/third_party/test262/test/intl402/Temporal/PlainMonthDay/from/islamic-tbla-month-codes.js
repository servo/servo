// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all month codes (M01-M12) in Islamic-tbla calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test that all month codes M01-M12 are valid for the Islamic-tbla calendar
// Islamic calendar month lengths:
// - 30 days: M01, M03, M05, M07, M09, M11, M12
// - 29 days: M02, M04, M06, M08, M10

const calendar = "islamic-tbla";

// Months with 30 days
const monthsWith30Days = [
  ["M01"],
  ["M03"],
  ["M05"],
  ["M07"],
  ["M09"],
  ["M11"],
  ["M12", 1971],
];

for (const [monthCode, referenceYear30 = 1972] of monthsWith30Days) {
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

// Months with 29 days
const monthsWith29Days = ["M02", "M04", "M06", "M08", "M10"];

for (const monthCode of monthsWith29Days) {
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`);

  const pmd29 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 29 });
  TemporalHelpers.assertPlainMonthDay(pmd29, monthCode, 29, `${monthCode}-29`);

  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 30 },
    { overflow: "constrain" }
  );
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 29, `day 30 should be constrained to 29 for ${monthCode}`);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 }, { overflow: "reject" });
  }, `${monthCode} with day 30 should throw with reject overflow`);
}
