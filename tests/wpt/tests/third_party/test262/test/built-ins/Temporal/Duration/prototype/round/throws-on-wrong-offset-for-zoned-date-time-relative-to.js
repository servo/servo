// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: round() throws on wrong offset for ZonedDateTime relativeTo string
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);

assert.throws(RangeError, () => d.round({
  smallestUnit: "seconds",
  relativeTo: "1971-01-01T00:00+02:00[-00:44:30]"
}));
