// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Verify that the year is appropriately formatted as 4 or 6 digits
features: [Temporal]
---*/

function epochNsInYear(year) {
  // Return an epoch nanoseconds value near the middle of the given year
  const avgNsPerYear = 31_556_952_000_000_000n;
  return (year - 1970n) * avgNsPerYear + (avgNsPerYear / 2n);
}

let instance = new Temporal.Instant(epochNsInYear(-100000n));
assert.sameValue(instance.toString(), "-100000-07-01T21:30:36Z", "large negative year formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(-10000n));
assert.sameValue(instance.toString(), "-010000-07-01T21:30:36Z", "smallest 5-digit negative year formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(-9999n));
assert.sameValue(instance.toString(), "-009999-07-02T03:19:48Z", "largest 4-digit negative year formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(-1000n));
assert.sameValue(instance.toString(), "-001000-07-02T09:30:36Z", "smallest 4-digit negative year formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(-999n));
assert.sameValue(instance.toString(), "-000999-07-02T15:19:48Z", "largest 3-digit negative year formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(-1n));
assert.sameValue(instance.toString(), "-000001-07-02T15:41:24Z", "year -1 formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(0n));
assert.sameValue(instance.toString(), "0000-07-01T21:30:36Z", "year 0 formatted as 4-digit");

instance = new Temporal.Instant(epochNsInYear(1n));
assert.sameValue(instance.toString(), "0001-07-02T03:19:48Z", "year 1 formatted as 4-digit");

instance = new Temporal.Instant(epochNsInYear(999n));
assert.sameValue(instance.toString(), "0999-07-02T03:41:24Z", "largest 3-digit positive year formatted as 4-digit");

instance = new Temporal.Instant(epochNsInYear(1000n));
assert.sameValue(instance.toString(), "1000-07-02T09:30:36Z", "smallest 4-digit positive year formatted as 4-digit");

instance = new Temporal.Instant(epochNsInYear(9999n));
assert.sameValue(instance.toString(), "9999-07-02T15:41:24Z", "largest 4-digit positive year formatted as 4-digit");

instance = new Temporal.Instant(epochNsInYear(10000n));
assert.sameValue(instance.toString(), "+010000-07-01T21:30:36Z", "smallest 5-digit positive year formatted as 6-digit");

instance = new Temporal.Instant(epochNsInYear(100000n));
assert.sameValue(instance.toString(), "+100000-07-01T21:30:36Z", "large positive year formatted as 6-digit");
