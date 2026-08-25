// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: auto value for fractionalSecondDigits option
features: [BigInt, Temporal]
---*/

const tests = [
  [new Temporal.Instant(192_258_181_000_000_000n), "1976-02-04T05:03:01Z"],
  [new Temporal.Instant(0n), "1970-01-01T00:00:00Z"],
  [new Temporal.Instant(30_000_000_000n), "1970-01-01T00:00:30Z"],
  [new Temporal.Instant(30_123_400_000n), "1970-01-01T00:00:30.1234Z"],
];

for (const [instant, expected] of tests) {
  assert.sameValue(instant.toString(), expected, "default is to emit seconds and drop trailing zeroes");
  assert.sameValue(instant.toString({ fractionalSecondDigits: "auto" }), expected, "auto is the default");
}
