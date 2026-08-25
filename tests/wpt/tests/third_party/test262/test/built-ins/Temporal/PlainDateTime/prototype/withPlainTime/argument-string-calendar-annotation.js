// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["12:34:56.987654321[u-ca=iso8601]", "without time zone"],
  ["12:34:56.987654321[UTC][u-ca=iso8601]", "with time zone"],
  ["12:34:56.987654321[!u-ca=iso8601]", "with ! and no time zone"],
  ["12:34:56.987654321[UTC][!u-ca=iso8601]", "with ! and time zone"],
  ["T12:34:56.987654321[u-ca=iso8601]", "with T and no time zone"],
  ["T12:34:56.987654321[UTC][u-ca=iso8601]", "with T and time zone"],
  ["T12:34:56.987654321[!u-ca=iso8601]", "with T, !, and no time zone"],
  ["T12:34:56.987654321[UTC][!u-ca=iso8601]", "with T, !, and time zone"],
  ["1970-01-01T12:34:56.987654321[u-ca=iso8601]", "with date and no time zone"],
  ["1970-01-01T12:34:56.987654321[UTC][u-ca=iso8601]", "with date and time zone"],
  ["1970-01-01T12:34:56.987654321[!u-ca=iso8601]", "with !, date, and no time zone"],
  ["1970-01-01T12:34:56.987654321[UTC][!u-ca=iso8601]", "with !, date, and time zone"],
  ["12:34:56.987654321[u-ca=hebrew]", "calendar annotation ignored"],
  ["12:34:56.987654321[u-ca=unknown]", "calendar annotation ignored even if unknown calendar"],
  ["12:34:56.987654321[!u-ca=unknown]", "calendar annotation ignored even if unknown calendar with !"],
  ["1970-01-01T12:34:56.987654321[u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

const instance = new Temporal.PlainDateTime(1976, 11, 18, 15, 23);

tests.forEach(([arg, description]) => {
  const result = instance.withPlainTime(arg);

  TemporalHelpers.assertPlainDateTime(
    result,
    1976, 11, "M11", 18, 12, 34, 56, 987, 654, 321,
    `calendar annotation (${description})`
  );
});
