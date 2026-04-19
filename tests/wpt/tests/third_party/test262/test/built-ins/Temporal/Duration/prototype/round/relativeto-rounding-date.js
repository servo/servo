// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test that round() counts the correct number of days when rounding relative to a date.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const days = new Temporal.Duration(0, 0, 0, 45, 0, 0, 0, 0, 0, 0);
const yearAndHalf = new Temporal.Duration(0, 0, 0, 547, 12, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(days.round({
    relativeTo: new Temporal.PlainDate(2019, 1, 1),
    smallestUnit: "months"}), 0, 2, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(days.negated().round({
    relativeTo: new Temporal.PlainDate(2019, 2, 15),
    smallestUnit: "months"}), 0, -1, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(yearAndHalf.round({
    relativeTo: new Temporal.PlainDate(2018, 1, 1),
    smallestUnit: "years"}), 2, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(yearAndHalf.round({
    relativeTo: new Temporal.PlainDate(2018, 7, 1),
    smallestUnit: "years"}), 1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(yearAndHalf.round({
    relativeTo: new Temporal.PlainDate(2019, 1, 1),
    smallestUnit: "years"}), 1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(yearAndHalf.round({
    relativeTo: new Temporal.PlainDate(2020, 1, 1),
    smallestUnit: "years"}), 1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(yearAndHalf.round({
    relativeTo: new Temporal.PlainDate(2020, 7, 1),
    smallestUnit: "years"}), 2, 0, 0, 0, 0, 0, 0, 0, 0, 0);
