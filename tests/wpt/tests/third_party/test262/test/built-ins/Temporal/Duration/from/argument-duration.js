// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: A Duration object is copied, not returned directly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const orig = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 654, 321);
const result = Temporal.Duration.from(orig);

TemporalHelpers.assertDuration(
  result,
  1, 2, 3, 4, 5, 6, 7, 987, 654, 321,
  "Duration is copied"
);

assert.notSameValue(
  result,
  orig,
  "When a Duration is given, the returned value is not the original Duration"
);
