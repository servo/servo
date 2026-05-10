// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Specify behavior of Instant.since when largest specified unit is specified
includes: [temporalHelpers.js]
features: [Temporal, BigInt]
---*/
const thePast = new Temporal.Instant(1234567890123456789n);
const theFuture = new Temporal.Instant(2345678901234567890n);
TemporalHelpers.assertDuration(theFuture.since(thePast), 0, 0, 0, 0, 0, 0, 1111111011, 111, 111, 101, 'does not include higher units than necessary (largest unit unspecified)');
TemporalHelpers.assertDuration(theFuture.since(thePast, { largestUnit: 'hours' }), 0, 0, 0, 0, 308641, 56, 51, 111, 111, 101, 'does not include higher units than necessary (largest unit is hours)');
TemporalHelpers.assertDuration(theFuture.since(thePast, { largestUnit: 'minutes' }), 0, 0, 0, 0, 0, 18518516, 51, 111, 111, 101, 'does not include higher units than necessary (largest unit is minutes)');
TemporalHelpers.assertDuration(theFuture.since(thePast, { largestUnit: 'seconds' }), 0, 0, 0, 0, 0, 0, 1111111011, 111, 111, 101, 'does not include higher units than necessary (largest unit is seconds)');
TemporalHelpers.assertDuration(theFuture.since(thePast, { largestUnit: 'milliseconds' }), 0, 0, 0, 0, 0, 0, 0, 1111111011111, 111, 101, 'does not include higher units than necessary (largest unit is milliseconds)');
TemporalHelpers.assertDuration(theFuture.since(thePast, { largestUnit: 'microseconds' }), 0, 0, 0, 0, 0, 0, 0, 0, 1111111011111111, 101, 'does not include higher units than necessary (largest unit is microseconds)');
TemporalHelpers.assertDuration(theFuture.since(thePast, { largestUnit: 'nanoseconds' }), 0, 0, 0, 0, 0, 0, 0, 0, 0, 1111111011111111000, 'does not include higher units than necessary (largest unit is nanoseconds)');
