// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Annotation keys are lowercase-only
features: [Temporal]
---*/

const invalidStrings = [
  ["1970-01-01T00:00Z[U-CA=iso8601]", "invalid capitalized key"],
  ["1970-01-01T00:00Z[u-CA=iso8601]", "invalid partially-capitalized key"],
  ["1970-01-01T00:00Z[FOO=bar]", "invalid capitalized unrecognized key"],
];
const instance = new Temporal.Instant(0n);
invalidStrings.forEach(([arg, descr]) => {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `annotation keys must be lowercase: ${arg} - ${descr}`
  );
});
