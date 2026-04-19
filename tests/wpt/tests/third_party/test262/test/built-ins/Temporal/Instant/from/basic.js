// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Basic functionality of Temporal.Instant.from
features: [Temporal]
---*/

const baseValue = 217_178_580_000_000_000n;

let instant = Temporal.Instant.from("1976-11-18T15:23Z");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue,
  "ISO string with UTC designator and minutes precision"
);

instant = Temporal.Instant.from("1976-11-18T15:23:30Z");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue + 30_000_000_000n,
  "ISO string with UTC designator and seconds precision"
);

instant = Temporal.Instant.from("1976-11-18T15:23:30.123Z");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue + 30_123_000_000n,
  "ISO string with UTC designator and milliseconds precision"
);

instant = Temporal.Instant.from("1976-11-18T15:23:30.123456Z");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue + 30_123_456_000n,
  "ISO string with UTC designator and microseconds precision"
);

instant = Temporal.Instant.from("1976-11-18T15:23:30.123456789Z");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue + 30_123_456_789n,
  "ISO string with UTC designator and nanoseconds precision"
);

instant = Temporal.Instant.from("1976-11-18T15:23-01:00");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue + 3600_000_000_000n,
  "ISO string with negative UTC offset"
);

instant = Temporal.Instant.from("1976-11-18T15:23+01:00");
assert.sameValue(
  instant.epochNanoseconds,
  baseValue - 3600_000_000_000n,
  "ISO string with positive UTC offset"
);
