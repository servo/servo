// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration
description: Constructor with mixed signs.
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.Duration(-1, 1, 1, 1, 1, 1, 1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, -1, 1, 1, 1, 1, 1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, -1, 1, 1, 1, 1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, -1, 1, 1, 1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, 1, -1, 1, 1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, 1, 1, -1, 1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, 1, 1, 1, -1, 1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, 1, 1, 1, 1, -1, 1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, 1, 1, 1, 1, 1, -1, 1));
assert.throws(RangeError, () => new Temporal.Duration(1, 1, 1, 1, 1, 1, 1, 1, 1, -1));
