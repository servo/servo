// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Tests calculations with roundingMode "ceil".
features: [Temporal]
---*/

const instance = new Temporal.Instant(217175010_123_987_500n /* 1976-11-18T14:23:30.1239875Z */);

const expected = [
  ["hour", 217177200_000_000_000n /* 1976-11-18T15:00:00Z */],
  ["minute", 217175040_000_000_000n /* 1976-11-18T14:24:00Z */],
  ["second", 217175011_000_000_000n /* 1976-11-18T14:23:31Z */],
  ["millisecond", 217175010_124_000_000n /* 1976-11-18T14:23:30.124Z */],
  ["microsecond", 217175010_123_988_000n /* 1976-11-18T14:23:30.123988Z */],
  ["nanosecond", 217175010_123_987_500n /* 1976-11-18T14:23:30.1239875Z */],
];

const roundingMode = "ceil";

expected.forEach(([smallestUnit, expected]) => {
  assert.sameValue(
    instance.round({ smallestUnit, roundingMode }).epochNanoseconds,
    expected,
    `rounds to ${smallestUnit} (roundingMode = ${roundingMode})`
  );
});
