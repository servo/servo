// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: relativeTo is required for rounding durations with calendar units
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);

assert.throws(RangeError, () => d.round({ largestUnit: "years" }));
assert.throws(RangeError, () => d.round({ largestUnit: "months" }));
assert.throws(RangeError, () => d.round({ largestUnit: "weeks" }));
assert.throws(RangeError, () => d.round({ largestUnit: "days" }));
assert.throws(RangeError, () => d.round({ largestUnit: "hours" }));
assert.throws(RangeError, () => d.round({ largestUnit: "minutes" }));
assert.throws(RangeError, () => d.round({ largestUnit: "seconds" }));
assert.throws(RangeError, () => d.round({ largestUnit: "milliseconds" }));
assert.throws(RangeError, () => d.round({ largestUnit: "microseconds" }));
assert.throws(RangeError, () => d.round({ largestUnit: "nanoseconds" }));
