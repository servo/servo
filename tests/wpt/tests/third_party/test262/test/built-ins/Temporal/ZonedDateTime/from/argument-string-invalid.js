// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Check that UTC offsets are parsed properly
features: [Temporal]
---*/

const invalidStrings = [
  // UTCOffset must be of the form hh:mm:ss or hhmmss");
  "2025-01-01T00:00:00+00:0000[UTC]",
  "2025-01-01T00:00:00+0000:00[UTC]",
  // Invalid date or time components, valid offset
  "202501-01T00:00:00+00:00[UTC]",
  "2025-0101T00:00:00+00:00[UTC]",
  "2025-01-01T00:0000+00:00[UTC]",
  "2025-01-01T0000:00+00:00[UTC]",
];

invalidStrings.forEach((s) => {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.from(s),
    `invalid date-time string (${s})`
  );
});

