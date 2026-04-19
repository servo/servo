// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: Basic usage
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const one = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
const two = new Temporal.PlainTime(14, 23, 30, 123, 456, 789);
const three = new Temporal.PlainTime(13, 30, 30, 123, 456, 789);

TemporalHelpers.assertDuration(one.since(two),
  0, 0, 0, 0, /* hours = */ 1, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(two.since(one),
  0, 0, 0, 0, /* hours = */ -1, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(one.since(three),
  0, 0, 0, 0, /* hours = */ 1, 53, 0, 0, 0, 0);
TemporalHelpers.assertDuration(three.since(one),
  0, 0, 0, 0, /* hours = */ -1, -53, 0, 0, 0, 0);
