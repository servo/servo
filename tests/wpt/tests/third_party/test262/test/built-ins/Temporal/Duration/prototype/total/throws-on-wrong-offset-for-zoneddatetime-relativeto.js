// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Throws on wrong offset for ZonedDateTime relativeTo string.
features: [Temporal]
---*/

const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 2, 31, 0);

assert.throws(RangeError, () => d.total({
  unit: "months",
  relativeTo: "1971-01-01T00:00+02:00[-00:44:30]"
}));
