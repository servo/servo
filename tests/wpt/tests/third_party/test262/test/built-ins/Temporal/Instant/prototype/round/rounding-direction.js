// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Rounding down is towards the Big Bang, not the epoch or 1 BCE
features: [Temporal]
---*/

const instance = new Temporal.Instant(-65_261_246_399_500_000_000n);  // -000099-12-15T12:00:00.5Z
assert.sameValue(
  instance.round({ smallestUnit: "second", roundingMode: "floor" }).epochNanoseconds,
  -65_261_246_400_000_000_000n,  // -000099-12-15T12:00:00Z
  "Rounding down is towards the Big Bang, not the epoch or 1 BCE (roundingMode floor)"
);
assert.sameValue(
  instance.round({ smallestUnit: "second", roundingMode: "trunc" }).epochNanoseconds,
  -65_261_246_400_000_000_000n,  // -000099-12-15T12:00:00Z
  "Rounding down is towards the Big Bang, not the epoch or 1 BCE (roundingMode trunc)"
);
assert.sameValue(
  instance.round({ smallestUnit: "second", roundingMode: "ceil" }).epochNanoseconds,
  -65_261_246_399_000_000_000n,  // -000099-12-15T12:00:01Z
  "Rounding up is away from the Big Bang, not the epoch or 1 BCE (roundingMode ceil)"
);
assert.sameValue(
  instance.round({ smallestUnit: "second", roundingMode: "halfExpand" }).epochNanoseconds,
  -65_261_246_399_000_000_000n,  // -000099-12-15T12:00:01Z
  "Rounding up is away from the Big Bang, not the epoch or 1 BCE (roundingMode halfExpand)"
);
