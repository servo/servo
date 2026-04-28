// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Difference between equivalent objects returns blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const d1 = new Temporal.ZonedDateTime(1n, "UTC");
const d2 = new Temporal.ZonedDateTime(1n, "UTC");
const result = d1.since(d2);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "blank result");
