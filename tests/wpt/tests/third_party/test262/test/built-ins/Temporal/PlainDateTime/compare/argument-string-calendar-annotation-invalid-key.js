// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Annotation keys are lowercase-only 
features: [Temporal]
---*/

const invalidStrings = [
  ["1970-01-01T00:00[U-CA=iso8601]", "invalid capitalized key"],
  ["1970-01-01T00:00[u-CA=iso8601]", "invalid partially-capitalized key"],
  ["1970-01-01T00:00[FOO=bar]", "invalid capitalized unrecognized key"],
];

invalidStrings.forEach(([arg, descr]) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.compare(arg, new Temporal.PlainDateTime(1976, 11, 18)),
    `annotation keys must be lowercase: ${arg} - ${descr} (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.compare(new Temporal.PlainDateTime(1976, 11, 18), arg),
    `annotation keys must be lowercase: ${arg} - ${descr} (second argument)`
  );
});
