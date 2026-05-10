// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2000-05-02[u-ca=iso8601]", "without time or time zone"],
  ["2000-05-02[UTC][u-ca=iso8601]", "with time zone and no time"],
  ["2000-05-02T15:23[u-ca=iso8601]", "without time zone"],
  ["2000-05-02T15:23[UTC][u-ca=iso8601]", "with time zone"],
  ["2000-05-02T15:23[!u-ca=iso8601]", "with ! and no time zone"],
  ["2000-05-02T15:23[UTC][!u-ca=iso8601]", "with ! and time zone"],
  ["2000-05-02T15:23[u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

const instance = new Temporal.PlainDate(2000, 5, 2);

tests.forEach(([arg, description]) => {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `calendar annotation (${description})`
  );
});
