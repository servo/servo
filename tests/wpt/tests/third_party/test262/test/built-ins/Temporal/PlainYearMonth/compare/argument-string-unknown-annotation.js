// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Various forms of unknown annotation
features: [Temporal]
---*/

const tests = [
  ["2019-12-15T15:23[foo=bar]", "alone"],
  ["2019-12-15T15:23[UTC][foo=bar]", "with time zone"],
  ["2019-12-15T15:23[u-ca=iso8601][foo=bar]", "with calendar"],
  ["2019-12-15T15:23[UTC][foo=bar][u-ca=iso8601]", "with time zone and calendar"],
  ["2019-12-15T15:23[foo=bar][_foo-bar0=Ignore-This-999999999999]", "with another unknown annotation"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainYearMonth.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `unknown annotation (${description})`
  );
});
