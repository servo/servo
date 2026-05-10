// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: A PlainTime object is copied, not returned directly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const orig = new Temporal.PlainTime(11, 42, 0, 0, 0, 0);
const result = Temporal.PlainTime.from(orig);

TemporalHelpers.assertPlainTime(
  result,
  11, 42, 0, 0, 0, 0,
  "PlainTime is copied"
);

assert.notSameValue(
  result,
  orig,
  "When a PlainTime is given, the returned value is not the original PlainTime"
);
