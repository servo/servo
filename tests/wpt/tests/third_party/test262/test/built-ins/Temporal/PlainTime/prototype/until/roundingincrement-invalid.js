// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: Tests roundingIncrement restrictions.
features: [Temporal]
---*/

const earlier = Temporal.PlainTime.from("08:22:36.123456789");
const later = Temporal.PlainTime.from("12:39:40.987654321");

assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "hours", roundingIncrement: 11 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "minutes", roundingIncrement: 29 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "milliseconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "nanoseconds", roundingIncrement: 29 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "hours", roundingIncrement: 24 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "minutes", roundingIncrement: 60 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 60 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "milliseconds", roundingIncrement: 1000 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 1000 }));
assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "nanoseconds", roundingIncrement: 1000 }));
