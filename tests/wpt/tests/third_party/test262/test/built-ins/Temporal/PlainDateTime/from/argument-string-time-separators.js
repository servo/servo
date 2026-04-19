// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Time separator in string argument can vary
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1976-11-18T15:23", "uppercase T"],
  ["1976-11-18t15:23", "lowercase T"],
  ["1976-11-18 15:23", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainDateTime.from(arg);

  TemporalHelpers.assertPlainDateTime(
    result,
    1976, 11, "M11", 18, 15, 23, 0, 0, 0, 0,
    `variant time separators (${description})`
  );
});
