// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: More than one time zone annotation is not syntactical
features: [Temporal]
---*/

const invalidStrings = [
  "1970-01-01T00:00Z[UTC][UTC]",
  "1970-01-01T00:00Z[!UTC][UTC]",
  "1970-01-01T00:00Z[UTC][!UTC]",
  "1970-01-01T00:00Z[UTC][u-ca=iso8601][UTC]",
  "1970-01-01T00:00Z[UTC][foo=bar][UTC]",
];

const epoch = new Temporal.Instant(0n);

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(arg, epoch),
    `reject more than one time zone annotation: ${arg} (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(epoch, arg),
    `reject more than one time zone annotation: ${arg} (second argument)`
  );
});
