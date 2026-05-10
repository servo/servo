// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
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
    () => Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6)),
    `annotation keys must be lowercase: ${arg} - ${descr} (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg),
    `annotation keys must be lowercase: ${arg} - ${descr} (second argument)`
  );
});
