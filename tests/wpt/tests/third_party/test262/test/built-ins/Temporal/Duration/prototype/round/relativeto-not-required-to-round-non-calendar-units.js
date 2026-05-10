// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: relativeTo is not required to round non-calendar units in durations without calendar units.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d2 = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);

// String params

TemporalHelpers.assertDuration(d2.round("days"),         0, 0, 0, 5, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round("hours"),        0, 0, 0, 5, 5, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round("minutes"),      0, 0, 0, 5, 5, 5, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round("seconds"),      0, 0, 0, 5, 5, 5, 5, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round("milliseconds"), 0, 0, 0, 5, 5, 5, 5, 5, 0, 0);
TemporalHelpers.assertDuration(d2.round("microseconds"), 0, 0, 0, 5, 5, 5, 5, 5, 5, 0);
TemporalHelpers.assertDuration(d2.round("nanoseconds"),  0, 0, 0, 5, 5, 5, 5, 5, 5, 5);

// Object params

TemporalHelpers.assertDuration(d2.round({ smallestUnit: "days" }),         0, 0, 0, 5, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round({ smallestUnit: "hours" }),        0, 0, 0, 5, 5, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round({ smallestUnit: "minutes" }),      0, 0, 0, 5, 5, 5, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round({ smallestUnit: "seconds" }),      0, 0, 0, 5, 5, 5, 5, 0, 0, 0);
TemporalHelpers.assertDuration(d2.round({ smallestUnit: "milliseconds" }), 0, 0, 0, 5, 5, 5, 5, 5, 0, 0);
TemporalHelpers.assertDuration(d2.round({ smallestUnit: "microseconds" }), 0, 0, 0, 5, 5, 5, 5, 5, 5, 0);
TemporalHelpers.assertDuration(d2.round({ smallestUnit: "nanoseconds" }),  0, 0, 0, 5, 5, 5, 5, 5, 5, 5);

