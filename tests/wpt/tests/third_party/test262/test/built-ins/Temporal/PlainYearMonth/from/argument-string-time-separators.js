// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Time separator in string argument can vary
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2019-12-15T15:23", "uppercase T"],
  ["2019-12-15t15:23", "lowercase T"],
  ["2019-12-15 15:23", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainYearMonth.from(arg);

  TemporalHelpers.assertPlainYearMonth(
    result,
    2019, 12, "M12",
    `variant time separators (${description})`
  );
});
