// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Tests for compare() with each possible outcome
features: [Temporal]
---*/

const tz1 = "UTC";
const tz2 = "-00:30";
const cal1 = "iso8601";
const cal2 = "gregory";

assert.sameValue(
  Temporal.ZonedDateTime.compare(new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, tz1, cal1), new Temporal.ZonedDateTime(500_000_000_000_000_000n, tz2, cal2)),
  1,
  ">"
);
assert.sameValue(
  Temporal.ZonedDateTime.compare(new Temporal.ZonedDateTime(-1000n, tz1, cal1), new Temporal.ZonedDateTime(1000n, tz2, cal2)),
  -1,
  "<"
);
assert.sameValue(
  Temporal.ZonedDateTime.compare(new Temporal.ZonedDateTime(123_456_789n, tz1, cal1), new Temporal.ZonedDateTime(123_456_789n, tz2, cal2)),
  0,
  "="
);
