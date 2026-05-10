// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: relativeTo object must contain at least the required correctly-spelled properties
features: [Temporal]
---*/

const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 2, 31, 0);

assert.throws(TypeError, () => d.total({
  unit: "months",
  relativeTo: {}
}));
assert.throws(TypeError, () => d.total({
  unit: "months",
  relativeTo: {
    years: 2020,
    month: 1,
    day: 1
  }
}));
