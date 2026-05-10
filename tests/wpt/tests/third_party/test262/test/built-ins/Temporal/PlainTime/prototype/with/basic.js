// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: Basic tests for with().
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
TemporalHelpers.assertPlainTime(plainTime, 15, 23, 30, 123, 456, 789, "initial");

const hour = plainTime.with({ hour: 3 });
TemporalHelpers.assertPlainTime(hour, 3, 23, 30, 123, 456, 789, "hour");

const minute = plainTime.with({ minute: 3 });
TemporalHelpers.assertPlainTime(minute, 15, 3, 30, 123, 456, 789, "minute");

const second = plainTime.with({ second: 3 });
TemporalHelpers.assertPlainTime(second, 15, 23, 3, 123, 456, 789, "second");

const millisecond = plainTime.with({ millisecond: 3 });
TemporalHelpers.assertPlainTime(millisecond, 15, 23, 30, 3, 456, 789, "millisecond");

const microsecond = plainTime.with({ microsecond: 3 });
TemporalHelpers.assertPlainTime(microsecond, 15, 23, 30, 123, 3, 789, "microsecond");

const nanosecond = plainTime.with({ nanosecond: 3 });
TemporalHelpers.assertPlainTime(nanosecond, 15, 23, 30, 123, 456, 3, "nanosecond");

const combined = plainTime.with({ minute: 8, nanosecond: 3 });
TemporalHelpers.assertPlainTime(combined, 15, 8, 30, 123, 456, 3, "combined");

const plural = plainTime.with({ minutes: 8, nanosecond: 3 });
TemporalHelpers.assertPlainTime(plural, 15, 23, 30, 123, 456, 3, "plural");
