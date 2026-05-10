// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
features: [Temporal, Intl.Era-monthcode]
description: >
  * Verify constrain behaviour correct when day >= 30 and monthCode indicates a leap month in which day 30 has never occurred
  * Verify constrain behaviour correct when day === 29 and monthCode indicates a leap month that never occurs.

info: |
  4.1.21 NonISOMonthDayToISOReferenceDate ( calendar, fields, overflow )
  ...
  4. If calendar is "chinese" or "dangi", then
    ...
    c. If fields.[[Day]] > daysInMonth, then
      i. If overflow is reject, throw a RangeError exception.
      ii. Let day be daysInMonth
    ...
    f. Let row be the row in Table 6 with a value in the "Month Code" column matching monthCode
    g. If the "Reference Year (Days 1-29)" column of row is "—", or day = 30 and the "Reference Year (Day 30)" column of row is "—", then
      i. If overflow is reject, throw a RangeError exception.
      ii. Set monthCode to CreateMonthCode(! ParseMonthCode(monthCode).[[MonthNumber]], false).

includes: [temporalHelpers.js]
---*/

// day >= 30 in leap months where day 30 doesn't exist, should constrain to day 30 of non-leap month and use correct reference year


const monthCodesWithYears = [
  { monthCode: "M01L", referenceYear: 1970 },
  { monthCode: "M02L", referenceYear: 1972 },
  { monthCode: "M08L", referenceYear: 1971 },
  { monthCode: "M09L", referenceYear: 1972 },
  { monthCode: "M10L", referenceYear: 1972 },
  { monthCode: "M11L", referenceYear: 1970 },
  { monthCode: "M12L", referenceYear: 1972 },
];

const daysToTest = [ 30, 31 ];

const calendars = [ "chinese", "dangi" ];

// constrain days >= 30 of leap months without day 30 to day 30 of corresponding non-leap month
for (let calendar of calendars){
  for (let {monthCode, referenceYear} of monthCodesWithYears) {
    for (let day of daysToTest) {
      const nonLeapMonthCode = monthCode.slice(0, -1);
      const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day});
      const constrain = Temporal.PlainMonthDay.from({ calendar, monthCode: nonLeapMonthCode, day: 30 });
      //TemporalHelpers.assertPlainMonthDay(constrain, nonLeapMonthCode, 30, `${nonLeapMonthCode} ${day} (constrained)`, referenceYear);
      assert.sameValue(constrain.equals(pmd), true);
    }
  }
}

// constrain day 29 of leap months that never occur to day 29 of corresponding non-leap month
const nonexistentMonthCodesWithYears = [
  { monthCode: "M01L", referenceYear: 1972 },
  { monthCode: "M12L", referenceYear: 1972 },
];

for (let calendar of calendars){
  for (let {monthCode, referenceYear} of nonexistentMonthCodesWithYears) {
    const nonLeapMonthCode = monthCode.slice(0, -1);
    const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 29});
    const constrain = Temporal.PlainMonthDay.from({ calendar, monthCode: nonLeapMonthCode, day: 29 });
     TemporalHelpers.assertPlainMonthDay(constrain, nonLeapMonthCode, 29, `${nonLeapMonthCode} 29 (constrained)`, referenceYear);
     assert.sameValue(constrain.equals(pmd), true);
  }
}

