// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Objects of a subclass are never created as return values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.Instant,
  [10n],
  "round",
  [{ smallestUnit: 'second', roundingMode: 'ceil' }],
  (result) => {
    assert.sameValue(result.epochNanoseconds, 1_000_000_000n, "epochNanoseconds result");
  },
);
