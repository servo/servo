// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: round() accepts plural units.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const inst = new Temporal.Instant(1_000_000_000_123_456_789n);

[
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond"
].forEach(smallestUnit => {
    TemporalHelpers.assertInstantsEqual(inst.round({ smallestUnit }),
                                        inst.round({ smallestUnit: `${ smallestUnit }s` }));
});
