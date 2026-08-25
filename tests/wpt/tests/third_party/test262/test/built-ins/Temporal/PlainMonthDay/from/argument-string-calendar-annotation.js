// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1976-05-02[u-ca=iso8601]", "without time or time zone"],
  ["1976-05-02T15:23[u-ca=iso8601]", "without time zone"],
  ["1976-05-02T15:23[UTC][u-ca=iso8601]", "with time zone"],
  ["1976-05-02T15:23[!u-ca=iso8601]", "with ! and no time zone"],
  ["1976-05-02T15:23[UTC][!u-ca=iso8601]", "with ! and time zone"],
  ["1976-05-02T15:23[u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainMonthDay.from(arg);

  TemporalHelpers.assertPlainMonthDay(
    result,
    "M05", 2,
    `calendar annotation (${description})`
  );
});
