// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
features: [Temporal, Intl.Era-monthcode]
description: Check correct results for 30-day leap months
includes: [temporalHelpers.js]
---*/

// Reference year for day 30 of leap months
//
// Month -> ISO year
//
// M01L     -
// M02L     -
// M03L     1955
// M04L     1944
// M05L     1952
// M06L     1941
// M07L     1938
// M08L     -
// M09L     -
// M10L     -
// M11L     -
// M12L     -
//
// M02L and M08L with 29 days are common, but with 30 are actually rather
// uncommon and are not known to have occurred in the range in which the Chinese
// calendar can be accurately calculated.
//
// See also "The Mathematics of the Chinese Calendar", Table 21 [1] for a
// distribution of leap months.
//
// [1] https://www.xirugu.com/CHI500/Dates_Time/Chinesecalender.pdf

const monthCodesWithYears = [
  { monthCode: "M03L", referenceYear: 1955 },
  { monthCode: "M04L", referenceYear: 1944 },
  { monthCode: "M05L", referenceYear: 1952 },
  { monthCode: "M06L", referenceYear: 1941 },
  { monthCode: "M07L", referenceYear: 1938 }
];

const calendar = "chinese";

for (let {monthCode, referenceYear} of monthCodesWithYears) {
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 30, `${monthCode}-30`, referenceYear);

  const constrain = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 });
  TemporalHelpers.assertPlainMonthDay(constrain, monthCode, 30, `${monthCode} (constrained)`, referenceYear);
  assert.sameValue(constrain.equals(pmd), true);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  });
}
