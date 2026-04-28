// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
description: Plain object arguments are supported.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
TemporalHelpers.assertPlainTime(plainTime.add({ hours: 16 }),
  7, 23, 30, 123, 456, 789, "add 16 hours across midnight boundary");
TemporalHelpers.assertPlainTime(plainTime.add({ minutes: 45 }),
  16, 8, 30, 123, 456, 789, "add 45 minutes");
TemporalHelpers.assertPlainTime(plainTime.add({ seconds: 800 }),
  15, 36, 50, 123, 456, 789, "add 800 seconds");
TemporalHelpers.assertPlainTime(plainTime.add({ milliseconds: 800 }),
  15, 23, 30, 923, 456, 789, "add 800 milliseconds");
TemporalHelpers.assertPlainTime(plainTime.add({ microseconds: 800 }),
  15, 23, 30, 124, 256, 789, "add 800 microseconds");
TemporalHelpers.assertPlainTime(plainTime.add({ nanoseconds: 300 }),
  15, 23, 30, 123, 457, 89, "add 300 nanoseconds");
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("07:23:30.123456789").add({ hours: -16 }),
  15, 23, 30, 123, 456, 789, "add -16 hours across midnight boundary");
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("16:08:30.123456789").add({ minutes: -45 }),
  15, 23, 30, 123, 456, 789, "add -45 minutes");
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("15:36:50.123456789").add({ seconds: -800 }),
  15, 23, 30, 123, 456, 789, "add -800 seconds");
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("15:23:30.923456789").add({ milliseconds: -800 }),
  15, 23, 30, 123, 456, 789, "add -800 milliseconds");
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("15:23:30.124256789").add({ microseconds: -800 }),
  15, 23, 30, 123, 456, 789, "add -800 microseconds");
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("15:23:30.123457089").add({ nanoseconds: -300 }),
  15, 23, 30, 123, 456, 789, "add -300 nanoseconds");
TemporalHelpers.assertPlainTime(plainTime.add({ minute: 1, hours: 1 }),
  16, 23, 30, 123, 456, 789, "misspelled property is ignored");
