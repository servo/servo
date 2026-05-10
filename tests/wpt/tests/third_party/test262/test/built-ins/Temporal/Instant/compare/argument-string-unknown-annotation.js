// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Various forms of unknown annotation
features: [Temporal]
---*/

const tests = [
  ["1970-01-01T00:00Z[foo=bar]", "alone"],
  ["1970-01-01T00:00Z[UTC][foo=bar]", "with time zone"],
  ["1970-01-01T00:00Z[u-ca=iso8601][foo=bar]", "with calendar"],
  ["1970-01-01T00:00Z[UTC][foo=bar][u-ca=iso8601]", "with time zone and calendar"],
  ["1970-01-01T00:00Z[foo=bar][_foo-bar0=Ignore-This-999999999999]", "with another unknown annotation"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.Instant.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `unknown annotation (${description})`
  );
});
