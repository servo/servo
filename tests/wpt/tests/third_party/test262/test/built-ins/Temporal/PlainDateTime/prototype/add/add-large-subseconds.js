// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Adding unbalanced durations with large subsecond values to a date
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const pdt1 = new Temporal.PlainDateTime(2020, 2, 29, 0, 57, 27, 747, 612, 578);

TemporalHelpers.assertPlainDateTime(pdt1.add(Temporal.Duration.from({nanoseconds: Number.MAX_SAFE_INTEGER})),
                                    2020, 6, "M06", 12, 6, 57, 27, 2, 353, 569);
TemporalHelpers.assertPlainDateTime(pdt1.add(Temporal.Duration.from({nanoseconds: Number.MIN_SAFE_INTEGER})),
                                    2019, 11, "M11", 16, 18, 57, 28, 492, 871, 587);

TemporalHelpers.assertPlainDateTime(pdt1.add(Temporal.Duration.from({microseconds: Number.MAX_SAFE_INTEGER})),
                                    2305, 8, "M08", 4, 0, 45, 2, 488, 603, 578);
TemporalHelpers.assertPlainDateTime(pdt1.add(Temporal.Duration.from({microseconds: Number.MIN_SAFE_INTEGER})),
                                    1734, 9, "M09", 26, 1, 9, 53, 6, 621, 578);

assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({milliseconds: Number.MAX_SAFE_INTEGER})));
assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({milliseconds: Number.MIN_SAFE_INTEGER})));

assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({seconds: Number.MAX_SAFE_INTEGER})));
assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({seconds: Number.MIN_SAFE_INTEGER})));

const bigNumber = 9007199254740990976;

TemporalHelpers.assertPlainDateTime(pdt1.add(Temporal.Duration.from({nanoseconds: bigNumber})),
                                    2305, 8, "M08", 4, 0, 45, 2, 488, 603, 554);
TemporalHelpers.assertPlainDateTime(pdt1.add(Temporal.Duration.from({nanoseconds: -bigNumber})),
                                    1734, 9, "M09", 26, 1, 9, 53, 6, 621, 602);

assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({microseconds: bigNumber})));
assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({microseconds: -bigNumber})));

assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({milliseconds: bigNumber})));
assert.throws(RangeError, () => pdt1.add(Temporal.Duration.from({milliseconds: -bigNumber})));

const pdt2 = new Temporal.PlainDateTime(0, 1, 1);

TemporalHelpers.assertPlainDateTime(pdt2.add(Temporal.Duration.from({nanoseconds: bigNumber})),
                                    285, 6, "M06", 4, 23, 47, 34, 740, 990, 976);
TemporalHelpers.assertPlainDateTime(pdt2.add(Temporal.Duration.from({nanoseconds: -bigNumber})),
                                    -286, 7, "M07", 29, 0, 12, 25, 259, 9, 24);

assert.throws(RangeError, () => pdt2.add(Temporal.Duration.from({microseconds: bigNumber})));
assert.throws(RangeError, () => pdt2.add(Temporal.Duration.from({microseconds: -bigNumber})));

assert.throws(RangeError, () => pdt2.add(Temporal.Duration.from({milliseconds: bigNumber})));
assert.throws(RangeError, () => pdt2.add(Temporal.Duration.from({milliseconds: -bigNumber})));
