// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1970-01-01T00:00Z[u-ca=iso8601]", "without time zone"],
  ["1970-01-01T00:00Z[UTC][u-ca=gregory]", "with time zone"],
  ["1970-01-01T00:00Z[!u-ca=hebrew]", "with ! and no time zone"],
  ["1970-01-01T00:00Z[UTC][!u-ca=chinese]", "with ! and time zone"],
  ["1970-01-01T00:00Z[u-ca=discord]", "annotation is ignored"],
  ["1970-01-01T00:00Z[!u-ca=discord]", "annotation with ! is ignored"],
  ["1970-01-01T00:00Z[u-ca=iso8601][u-ca=discord]", "two annotations are ignored"],
];

const instance = new Temporal.Instant(0n);

tests.forEach(([arg, description]) => {
  const result = instance.until(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `calendar annotation (${description})`
  );
});
