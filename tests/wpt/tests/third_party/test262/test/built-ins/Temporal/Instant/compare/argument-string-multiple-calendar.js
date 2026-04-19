// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: >
  More than one calendar annotation is not syntactical if any have the criical
  flag
features: [Temporal]
---*/

const invalidStrings = [
  "1970-01-01T00:00Z[u-ca=iso8601][!u-ca=iso8601]",
  "1970-01-01T00:00Z[!u-ca=iso8601][u-ca=iso8601]",
  "1970-01-01T00:00Z[UTC][u-ca=iso8601][!u-ca=iso8601]",
  "1970-01-01T00:00Z[u-ca=iso8601][foo=bar][!u-ca=iso8601]",
];

const epoch = new Temporal.Instant(0n);

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(arg, epoch),
    `reject more than one calendar annotation if any critical: ${arg} (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(epoch, arg),
    `reject more than one calendar annotation if any critical: ${arg} (second argument)`
  );
});
