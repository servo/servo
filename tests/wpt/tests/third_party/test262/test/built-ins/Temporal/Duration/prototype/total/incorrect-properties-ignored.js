// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Incorrectly-spelled properties are ignored in relativeTo
features: [Temporal]
---*/

const oneMonth = new Temporal.Duration(0, 0, 0, 31, 0, 0, 0, 0, 0, 0);

assert.sameValue(oneMonth.total({
  unit: "months",
  relativeTo: {
    year: 2020,
    month: 1,
    day: 1,
    months: 2
  }
}), 1);
