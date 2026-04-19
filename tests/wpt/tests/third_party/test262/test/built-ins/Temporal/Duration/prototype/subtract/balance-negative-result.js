// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: A negative duration result is balanced correctly by the modulo operation in NanosecondsToDays
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration1 = new Temporal.Duration(0, 0, 0, 0, -60);
const duration2 = new Temporal.Duration(0, 0, 0, -1);

const result = duration1.subtract(duration2);
TemporalHelpers.assertDuration(result, 0, 0, 0, -1, -12, 0, 0, 0, 0, 0);
