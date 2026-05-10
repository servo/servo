// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: See https://github.com/tc39/proposal-temporal/issues/3168
includes: [temporalHelpers.js]
features: [Temporal]
---*/

var d = new Temporal.Duration(1, 0, 0, 0, 1);
var relativeTo = new Temporal.PlainDate(2020, 2, 29);
TemporalHelpers.assertDuration(d.round({ smallestUnit: 'years', relativeTo }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

d = new Temporal.Duration(0, 1, 0, 0, 10);
relativeTo = new Temporal.PlainDate(2020, 1, 31);
TemporalHelpers.assertDuration(d.round({ smallestUnit: 'months', roundingMode: 'expand', relativeTo }),
  0, 2, 0, 0, 0, 0, 0, 0, 0, 0);

d = new Temporal.Duration(2345, 0, 0, 0, 12);
relativeTo = new Temporal.PlainDate(2020, 2, 29)
TemporalHelpers.assertDuration(d.round({ smallestUnit: 'years', roundingMode: 'expand', relativeTo }),
  2346, 0, 0, 0, 0, 0, 0, 0, 0, 0);

d = new Temporal.Duration(1);
relativeTo = new Temporal.PlainDate(2020, 2, 29)
TemporalHelpers.assertDuration(d.round({ smallestUnit: 'months', relativeTo }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0);
