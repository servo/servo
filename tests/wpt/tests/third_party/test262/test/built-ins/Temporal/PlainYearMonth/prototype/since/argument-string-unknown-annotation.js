// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Various forms of unknown annotation
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2019-12-15T15:23[foo=bar]", "alone"],
  ["2019-12-15T15:23[UTC][foo=bar]", "with time zone"],
  ["2019-12-15T15:23[u-ca=iso8601][foo=bar]", "with calendar"],
  ["2019-12-15T15:23[UTC][foo=bar][u-ca=iso8601]", "with time zone and calendar"],
  ["2019-12-15T15:23[foo=bar][_foo-bar0=Ignore-This-999999999999]", "with another unknown annotation"],
];

const instance = new Temporal.PlainYearMonth(2019, 12);

tests.forEach(([arg, description]) => {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `unknown annotation (${description})`
  );
});
