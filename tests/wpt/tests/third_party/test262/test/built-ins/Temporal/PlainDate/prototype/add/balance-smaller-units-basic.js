// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.calendar.prototype.dateadd
description: Durations with units smaller than days are balanced
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(1976, 11, 18);

// lower units that don't balance up to a day
TemporalHelpers.assertPlainDate(date.add({ hours: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.add({ minutes: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.add({ seconds: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.add({ milliseconds: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.add({ microseconds: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.add({ nanoseconds: 1 }), 1976, 11, "M11", 18);

// lower units that balance up to a day or more
TemporalHelpers.assertPlainDate(date.add({ hours: 24 }), 1976, 11, "M11", 19);
TemporalHelpers.assertPlainDate(date.add({ hours: 36 }), 1976, 11, "M11", 19);
TemporalHelpers.assertPlainDate(date.add({ hours: 48 }), 1976, 11, "M11", 20);
TemporalHelpers.assertPlainDate(date.add({ minutes: 1440 }), 1976, 11, "M11", 19);
TemporalHelpers.assertPlainDate(date.add({ seconds: 86400 }), 1976, 11, "M11", 19);
TemporalHelpers.assertPlainDate(date.add({ milliseconds: 86400_000 }), 1976, 11, "M11", 19);
TemporalHelpers.assertPlainDate(date.add({ microseconds: 86400_000_000 }), 1976, 11, "M11", 19);
TemporalHelpers.assertPlainDate(date.add({ nanoseconds: 86400_000_000_000 }), 1976, 11, "M11", 19);
