// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Difference between equivalent objects returns blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const d1 = new Temporal.PlainDateTime(2025, 8, 22, 13, 52);
const d2 = new Temporal.PlainDateTime(2025, 8, 22, 13, 52);
const result = d1.since(d2);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "blank result");
