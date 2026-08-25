// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: Hour argument defaults to 0 if not given
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const args = [2000, 5, 2];

TemporalHelpers.assertPlainDateTime(
  new Temporal.PlainDateTime(...args, undefined),
  2000, 5, "M05", 2, 0, 0, 0, 0, 0, 0,
  "hour default argument (argument present)"
);

TemporalHelpers.assertPlainDateTime(
  new Temporal.PlainDateTime(...args),
  2000, 5, "M05", 2, 0, 0, 0, 0, 0, 0,
  "hour default argument (argument missing)"
);
