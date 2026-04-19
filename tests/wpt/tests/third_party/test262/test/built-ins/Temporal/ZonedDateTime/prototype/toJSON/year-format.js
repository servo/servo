// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tojson
description: Verify that the year is appropriately formatted as 4 or 6 digits
features: [Temporal]
---*/

function epochNsInYear(year) {
  // Return an epoch nanoseconds value near the middle of the given year
  const avgNsPerYear = 31_556_952_000_000_000n;
  return (year - 1970n) * avgNsPerYear + (avgNsPerYear / 2n);
}

let instance = new Temporal.ZonedDateTime(epochNsInYear(-100000n), "UTC");
assert.sameValue(instance.toJSON(), "-100000-07-01T21:30:36+00:00[UTC]", "large negative year formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(-10000n), "UTC");
assert.sameValue(instance.toJSON(), "-010000-07-01T21:30:36+00:00[UTC]", "smallest 5-digit negative year formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(-9999n), "UTC");
assert.sameValue(instance.toJSON(), "-009999-07-02T03:19:48+00:00[UTC]", "largest 4-digit negative year formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(-1000n), "UTC");
assert.sameValue(instance.toJSON(), "-001000-07-02T09:30:36+00:00[UTC]", "smallest 4-digit negative year formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(-999n), "UTC");
assert.sameValue(instance.toJSON(), "-000999-07-02T15:19:48+00:00[UTC]", "largest 3-digit negative year formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(-1n), "UTC");
assert.sameValue(instance.toJSON(), "-000001-07-02T15:41:24+00:00[UTC]", "year -1 formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(0n), "UTC");
assert.sameValue(instance.toJSON(), "0000-07-01T21:30:36+00:00[UTC]", "year 0 formatted as 4-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(1n), "UTC");
assert.sameValue(instance.toJSON(), "0001-07-02T03:19:48+00:00[UTC]", "year 1 formatted as 4-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(999n), "UTC");
assert.sameValue(instance.toJSON(), "0999-07-02T03:41:24+00:00[UTC]", "largest 3-digit positive year formatted as 4-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(1000n), "UTC");
assert.sameValue(instance.toJSON(), "1000-07-02T09:30:36+00:00[UTC]", "smallest 4-digit positive year formatted as 4-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(9999n), "UTC");
assert.sameValue(instance.toJSON(), "9999-07-02T15:41:24+00:00[UTC]", "largest 4-digit positive year formatted as 4-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(10000n), "UTC");
assert.sameValue(instance.toJSON(), "+010000-07-01T21:30:36+00:00[UTC]", "smallest 5-digit positive year formatted as 6-digit");

instance = new Temporal.ZonedDateTime(epochNsInYear(100000n), "UTC");
assert.sameValue(instance.toJSON(), "+100000-07-01T21:30:36+00:00[UTC]", "large positive year formatted as 6-digit");
