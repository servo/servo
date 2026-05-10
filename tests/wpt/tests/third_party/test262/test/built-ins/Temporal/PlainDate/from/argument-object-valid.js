// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Property bag is correctly converted into PlainDate
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const valid = [
  [
    { year: 2019, month: 10, monthCode: "M10", day: 1, hour: 14, minute: 20, second: 36 },
    2019, 10, "M10", 1
  ],
  [
    { year: 1976, month: 11, day: 18 },
    1976, 11, "M11", 18
  ],
  [
    { year: 1976, monthCode: "M11", day: 18 },
    1976, 11, "M11", 18
  ],
  [
    { year: 1976, month: 11, day: 18, days: 15 },
    1976, 11, "M11", 18
  ],
];

for (const [dateTimeFields, ...expected] of valid) {
  const plainDate = Temporal.PlainDate.from(dateTimeFields);
  TemporalHelpers.assertPlainDate(plainDate, ...expected, `from(${JSON.stringify(dateTimeFields)}`);
}
