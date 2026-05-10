// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: relativeTo required to round durations with calendar units
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);

// relativeTo is required to round durations with calendar units (object param)
assert.throws(RangeError, () => d.total({ unit: "years" }));
assert.throws(RangeError, () => d.total({ unit: "months" }));
assert.throws(RangeError, () => d.total({ unit: "weeks" }));
assert.throws(RangeError, () => d.total({ unit: "days" }));
assert.throws(RangeError, () => d.total({ unit: "hours" }));
assert.throws(RangeError, () => d.total({ unit: "minutes" }));
assert.throws(RangeError, () => d.total({ unit: "seconds" }));
assert.throws(RangeError, () => d.total({ unit: "milliseconds" }));
assert.throws(RangeError, () => d.total({ unit: "microseconds" }));
assert.throws(RangeError, () => d.total({ unit: "nanoseconds" }));

// relativeTo is required to round durations with calendar units (string param)
assert.throws(RangeError, () => d.total("years"));
assert.throws(RangeError, () => d.total("months"));
assert.throws(RangeError, () => d.total("weeks"));
assert.throws(RangeError, () => d.total("days"));
assert.throws(RangeError, () => d.total("hours"));
assert.throws(RangeError, () => d.total("minutes"));
assert.throws(RangeError, () => d.total("seconds"));
assert.throws(RangeError, () => d.total("milliseconds"));
assert.throws(RangeError, () => d.total("microseconds"));
assert.throws(RangeError, () => d.total("nanoseconds"));
