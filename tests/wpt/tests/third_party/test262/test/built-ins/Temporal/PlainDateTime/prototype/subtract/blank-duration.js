// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(2025, 8, 22, 14, 1);
const blank = new Temporal.Duration();
const result = dt.subtract(blank);
TemporalHelpers.assertPlainDateTime(result, 2025, 8, "M08", 22, 14, 1, 0, 0, 0, 0, "result is unchanged");
