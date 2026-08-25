// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: relativeTo required to round calendar units even in durations without calendar units.
features: [Temporal]
---*/

const d2 = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);

// Object param
assert.throws(RangeError, () => d2.total({ unit: "years" }));
assert.throws(RangeError, () => d2.total({ unit: "months" }));
assert.throws(RangeError, () => d2.total({ unit: "weeks" }));

// String param
assert.throws(RangeError, () => d2.total("years"));
assert.throws(RangeError, () => d2.total("months"));
assert.throws(RangeError, () => d2.total("weeks"));
