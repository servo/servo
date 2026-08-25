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
TemporalHelpers.assertPlainDate(date.subtract({ hours: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.subtract({ minutes: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.subtract({ seconds: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.subtract({ milliseconds: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.subtract({ microseconds: 1 }), 1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.subtract({ nanoseconds: 1 }), 1976, 11, "M11", 18);

// lower units that balance up to a day or more
TemporalHelpers.assertPlainDate(date.subtract({ hours: 24 }), 1976, 11, "M11", 17);
TemporalHelpers.assertPlainDate(date.subtract({ hours: 36 }), 1976, 11, "M11", 17);
TemporalHelpers.assertPlainDate(date.subtract({ hours: 48 }), 1976, 11, "M11", 16);
TemporalHelpers.assertPlainDate(date.subtract({ minutes: 1440 }), 1976, 11, "M11", 17);
TemporalHelpers.assertPlainDate(date.subtract({ seconds: 86400 }), 1976, 11, "M11", 17);
TemporalHelpers.assertPlainDate(date.subtract({ milliseconds: 86400_000 }), 1976, 11, "M11", 17);
TemporalHelpers.assertPlainDate(date.subtract({ microseconds: 86400_000_000 }), 1976, 11, "M11", 17);
TemporalHelpers.assertPlainDate(date.subtract({ nanoseconds: 86400_000_000_000 }), 1976, 11, "M11", 17);
