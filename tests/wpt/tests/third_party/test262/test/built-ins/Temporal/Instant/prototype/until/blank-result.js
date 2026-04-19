// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Difference between equivalent objects returns blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const i1 = new Temporal.Instant(1n);
const i2 = new Temporal.Instant(1n);
const result = i1.until(i2);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "blank result");
