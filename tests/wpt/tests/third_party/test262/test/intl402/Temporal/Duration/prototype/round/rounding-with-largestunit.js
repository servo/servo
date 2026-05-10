// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.duration.prototype.round
description: Rounding mode is taken into account when largestUnit is present.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by Adam Shaw

const dur = new Temporal.Duration(0, 0, 0, 0, /* hours = */ 13, 0, 0, 0, 0, 0);
const zdt = new Temporal.ZonedDateTime(0n, "UTC");

TemporalHelpers.assertDuration(
    dur.round({
        relativeTo: zdt,
        largestUnit: 'hours',
        smallestUnit: 'hours',
        roundingIncrement: 12,
        roundingMode: 'ceil'
    }), 0, 0, 0, 0, 24, 0, 0, 0, 0, 0);
