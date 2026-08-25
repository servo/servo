// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Throws RangeError when the argument is an invalid string
features: [Temporal]
---*/

let t1 = new Temporal.PlainDateTime(2021, 3, 14, 1, 2, 3, 4, 5, 6);
assert.throws(RangeError, () => Temporal.PlainDateTime.compare(t1, 'invalid iso8601 string'), "invalid string in second argument");
assert.throws(RangeError, () => Temporal.PlainDateTime.compare('invalid iso8601 string', t1), "invalid string in first argument");
