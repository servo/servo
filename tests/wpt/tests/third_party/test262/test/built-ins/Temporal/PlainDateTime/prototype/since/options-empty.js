// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Empty objects are acceptable
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2021, 2, 1, 0, 0);

TemporalHelpers.assertDuration(
  feb21.since(feb20, {}),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "empty plain object options"
);

TemporalHelpers.assertDuration(
  feb21.since(feb20, () => {}),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "empty function object options"
);
