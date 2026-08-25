// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Tests for compare() with each possible outcome
features: [Temporal]
---*/

assert.sameValue(
  Temporal.Instant.compare(new Temporal.Instant(1_000_000_000_000_000_000n), new Temporal.Instant(500_000_000_000_000_000n)),
  1,
  ">"
);
assert.sameValue(
  Temporal.Instant.compare(new Temporal.Instant(-1000n), new Temporal.Instant(1000n)),
  -1,
  "<"
);
assert.sameValue(
  Temporal.Instant.compare(new Temporal.Instant(123_456_789n), new Temporal.Instant(123_456_789n)),
  0,
  "="
);
