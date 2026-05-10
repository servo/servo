// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
---*/

const tests = [
  ["1976-05-02[u-ca=iso8601]", "without time or time zone"],
  ["2019-12-15T15:23[u-ca=iso8601]", "without time zone"],
  ["2019-12-15T15:23[UTC][u-ca=iso8601]", "with time zone"],
  ["2019-12-15T15:23[!u-ca=iso8601]", "with ! and no time zone"],
  ["2019-12-15T15:23[UTC][!u-ca=iso8601]", "with ! and time zone"],
  ["2019-12-15T15:23[u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainYearMonth.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `calendar annotation (${description})`
  );
});
