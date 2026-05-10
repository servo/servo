// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Various forms of unknown annotation
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1976-05-02T15:23[foo=bar]", "alone"],
  ["1976-05-02T15:23[UTC][foo=bar]", "with time zone"],
  ["1976-05-02T15:23[u-ca=iso8601][foo=bar]", "with calendar"],
  ["1976-05-02T15:23[UTC][foo=bar][u-ca=iso8601]", "with time zone and calendar"],
  ["1976-05-02T15:23[foo=bar][_foo-bar0=Ignore-This-999999999999]", "with another unknown annotation"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainMonthDay.from(arg);

  TemporalHelpers.assertPlainMonthDay(
    result,
    "M05", 2,
    `unknown annotation (${description})`
  );
});
