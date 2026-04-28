// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.subtract
description: Subtracting unbalanced durations with large subsecond values from a time
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const t1 = new Temporal.PlainTime(0, 57, 27, 747, 612, 578);

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({nanoseconds: Number.MAX_SAFE_INTEGER})),
                                18, 57, 28, 492, 871, 587);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({nanoseconds: Number.MIN_SAFE_INTEGER})),
                                6, 57, 27, 2, 353, 569);

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({microseconds: Number.MAX_SAFE_INTEGER})),
                                1, 9, 53, 6, 621, 578);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({microseconds: Number.MIN_SAFE_INTEGER})),
                                0, 45, 2, 488, 603, 578);

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({milliseconds: Number.MAX_SAFE_INTEGER})),
                                15, 58, 26, 756, 612, 578);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({milliseconds: Number.MIN_SAFE_INTEGER})),
                                9, 56, 28, 738, 612, 578);

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({seconds: Number.MAX_SAFE_INTEGER})),
                                17, 20, 56, 747, 612, 578);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({seconds: Number.MIN_SAFE_INTEGER})),
                                8, 33, 58, 747, 612, 578);

const bigNumber = 9007199254740990976;

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({nanoseconds: bigNumber})),
                                1, 9, 53, 6, 621, 602);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({nanoseconds: -bigNumber})),
                                0, 45, 2, 488, 603, 554);

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({microseconds: bigNumber})),
                                15, 58, 26, 756, 636, 578);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({microseconds: -bigNumber})),
                                9, 56, 28, 738, 588, 578);

TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({milliseconds: bigNumber})),
                                17, 20, 56, 771, 612, 578);
TemporalHelpers.assertPlainTime(t1.subtract(Temporal.Duration.from({milliseconds: -bigNumber})),
                                8, 33, 58, 723, 612, 578);

const t2 = new Temporal.PlainTime(0);

TemporalHelpers.assertPlainTime(t2.subtract(Temporal.Duration.from({nanoseconds: bigNumber})),
                                0, 12, 25, 259, 9, 24);
TemporalHelpers.assertPlainTime(t2.subtract(Temporal.Duration.from({nanoseconds: -bigNumber})),
                                23, 47, 34, 740, 990, 976);

TemporalHelpers.assertPlainTime(t2.subtract(Temporal.Duration.from({microseconds: bigNumber})),
                                15, 0, 59, 9, 24, 0);
TemporalHelpers.assertPlainTime(t2.subtract(Temporal.Duration.from({microseconds: -bigNumber})),
                                8, 59, 0, 990, 976, 0);

TemporalHelpers.assertPlainTime(t2.subtract(Temporal.Duration.from({milliseconds: bigNumber})),
                                16, 23, 29, 24, 0, 0);
TemporalHelpers.assertPlainTime(t2.subtract(Temporal.Duration.from({milliseconds: -bigNumber})),
                                7, 36, 30, 976, 0, 0);
