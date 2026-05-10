// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: Basic usage
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const one = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
const two = new Temporal.PlainTime(16, 23, 30, 123, 456, 789);
const three = new Temporal.PlainTime(17, 0, 30, 123, 456, 789);

TemporalHelpers.assertDuration(one.until(two),
  0, 0, 0, 0, /* hours = */ 1, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(two.until(one),
  0, 0, 0, 0, /* hours = */ -1, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(one.until(three),
  0, 0, 0, 0, /* hours = */ 1, 37, 0, 0, 0, 0);
TemporalHelpers.assertDuration(three.until(one),
  0, 0, 0, 0, /* hours = */ -1, -37, 0, 0, 0, 0);
