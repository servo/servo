// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: auto value for fractionalSecondDigits option
features: [BigInt, Temporal]
---*/

const tests = [
  [new Temporal.ZonedDateTime(192_258_181_000_000_000n, "UTC"), "1976-02-04T05:03:01+00:00[UTC]"],
  [new Temporal.ZonedDateTime(0n, "UTC"), "1970-01-01T00:00:00+00:00[UTC]"],
  [new Temporal.ZonedDateTime(30_000_000_000n, "UTC"), "1970-01-01T00:00:30+00:00[UTC]"],
  [new Temporal.ZonedDateTime(30_123_400_000n, "UTC"), "1970-01-01T00:00:30.1234+00:00[UTC]"],
];

for (const [datetime, expected] of tests) {
  assert.sameValue(datetime.toString(), expected, "default is to emit seconds and drop trailing zeroes");
  assert.sameValue(datetime.toString({ fractionalSecondDigits: "auto" }), expected, "auto is the default");
}
