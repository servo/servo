// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Time separator in string argument can vary
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2000-05-02T15:23", "uppercase T"],
  ["2000-05-02t15:23", "lowercase T"],
  ["2000-05-02 15:23", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainDate.from(arg);

  TemporalHelpers.assertPlainDate(
    result,
    2000, 5, "M05", 2,
    `variant time separators (${description})`
  );
});
