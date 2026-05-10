// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2019-12-15[u-ca=iso8601]", "without time or time zone"],
  ["2019-12-15T15:23[u-ca=iso8601]", "without time zone"],
  ["2019-12-15T15:23[UTC][u-ca=iso8601]", "with time zone"],
  ["2019-12-15T15:23[!u-ca=iso8601]", "with ! and no time zone"],
  ["2019-12-15T15:23[UTC][!u-ca=iso8601]", "with ! and time zone"],
  ["2019-12-15T15:23[u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

const instance = new Temporal.PlainYearMonth(2019, 12);

tests.forEach(([arg, description]) => {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `calendar annotation (${description})`
  );
});
