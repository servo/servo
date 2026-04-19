// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Empty options are valid
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2021, 2, 1, 0, 0);

TemporalHelpers.assertDuration(feb20.until(feb21, {}),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "empty options (plain object) are acceptable");

TemporalHelpers.assertDuration(feb20.until(feb21, () => {}),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "empty options (function object) are acceptable");
