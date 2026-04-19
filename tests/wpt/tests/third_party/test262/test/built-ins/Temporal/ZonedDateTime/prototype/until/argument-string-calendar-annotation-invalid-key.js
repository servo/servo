// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Annotation keys are lowercase-only
features: [Temporal]
---*/

const invalidStrings = [
  ["1970-01-01T00:00[UTC][U-CA=iso8601]", "invalid capitalized key"],
  ["1970-01-01T00:00[UTC][u-CA=iso8601]", "invalid partially-capitalized key"],
  ["1970-01-01T00:00[UTC][FOO=bar]", "invalid capitalized unrecognized key"],
];
const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);
invalidStrings.forEach(([arg, descr]) => {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `annotation keys must be lowercase: ${arg} - ${descr}`
  );
});
