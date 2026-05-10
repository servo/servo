// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Throws RangeError when duration string is invalid
info: |
  ...
  5. Set duration to ? ToTemporalDuration(duration).
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);
assert.throws(RangeError,
  () => instance.add("invalid duration string"),
  "invalid duration string causes a RangeError");
