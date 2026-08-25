// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: Annotation keys are lowercase-only
features: [Temporal]
---*/

const invalidStrings = [
  ["1970-01-01[U-CA=iso8601]", "invalid capitalized key"],
  ["1970-01-01[u-CA=iso8601]", "invalid partially-capitalized key"],
  ["1970-01-01[FOO=bar]", "invalid capitalized unrecognized key"],
];
const instance = new Temporal.PlainDate(2000, 5, 2);
invalidStrings.forEach(([arg, descr]) => {
  assert.throws(
    RangeError,
    () => instance.equals(arg),
    `annotation keys must be lowercase: ${arg} - ${descr}`
  );
});
