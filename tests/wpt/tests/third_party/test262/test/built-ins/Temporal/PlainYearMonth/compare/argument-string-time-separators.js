// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Time separator in string argument can vary
features: [Temporal]
---*/

const yearMonth = new Temporal.PlainYearMonth(2019, 12);
const tests = [
  ["2019-12-15T15:23", "uppercase T"],
  ["2019-12-15t15:23", "lowercase T"],
  ["2019-12-15 15:23", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  assert.sameValue(
    Temporal.PlainYearMonth.compare(arg, yearMonth),
    0,
    `variant time separators (${description}), first argument`
  );

  assert.sameValue(
    Temporal.PlainYearMonth.compare(yearMonth, arg),
    0,
    `variant time separators (${description}), second argument`
  );
});
