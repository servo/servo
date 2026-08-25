// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Tests roundingIncrement restrictions.
features: [Temporal]
---*/

const plainTime = Temporal.PlainTime.from("08:22:36.123456789");

assert.throws(RangeError, () => plainTime.round({ smallestUnit: "hours", roundingIncrement: 11 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "minutes", roundingIncrement: 29 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "seconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "hours", roundingIncrement: 24 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "minutes", roundingIncrement: 60 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "seconds", roundingIncrement: 60 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 1000 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 1000 }));
assert.throws(RangeError, () => plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 1000 }));
