// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Annotation keys are lowercase-only 
features: [Temporal]
---*/

const invalidStrings = [
  ["1970-01-01T00:00[UTC][U-CA=iso8601]", "invalid capitalized key"],
  ["1970-01-01T00:00[UTC][u-CA=iso8601]", "invalid partially-capitalized key"],
  ["1970-01-01T00:00[UTC][FOO=bar]", "invalid capitalized unrecognized key"],
];

invalidStrings.forEach(([arg, descr]) => {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.from(arg),
    `annotation keys must be lowercase: ${arg} - ${descr}`
  );
  for (const offset of ["use", "prefer", "ignore", "reject"]) {
    assert.throws(
      RangeError,
      () => Temporal.ZonedDateTime.from(arg, { offset }),
      `annotation keys must be lowercase: ${arg} - ${descr} (offset ${offset})`
    );
  }
});
