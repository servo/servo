// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
features: [Temporal]
description: Check correct results for 30-day leap months
includes: [temporalHelpers.js]
---*/

// These reference years happen to be identical to the ones in
// chinese-30-day-leap-months.js.

const monthCodesWithYears = [
  { monthCode: "M03L", referenceYear: 1955 },
  { monthCode: "M04L", referenceYear: 1944 },
  { monthCode: "M05L", referenceYear: 1952 },
  { monthCode: "M06L", referenceYear: 1941 },
  { monthCode: "M07L", referenceYear: 1938 }
];

const calendar = "dangi";

// Months can have up to 30 days.
const day = 30;

for (let {monthCode, referenceYear} of monthCodesWithYears) {
  let pmd = Temporal.PlainMonthDay.from({calendar, monthCode, day});
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, day, monthCode, referenceYear);

  let constrain = Temporal.PlainMonthDay.from({calendar, monthCode, day: day + 1}, {overflow: "constrain"});
  TemporalHelpers.assertPlainMonthDay(constrain, monthCode, day, `${monthCode} (constrained)`, referenceYear);
  assert.sameValue(constrain.equals(pmd), true);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({calendar, monthCode, day: day + 1}, {overflow: "reject"});
  });
}

