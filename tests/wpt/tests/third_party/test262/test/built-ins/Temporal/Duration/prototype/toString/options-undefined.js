// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 650, 0);

const explicit = duration.toString(undefined);
assert.sameValue(explicit, "P1Y2M3W4DT5H6M7.98765S", "default precision is auto, and rounding is trunc");

const implicit = duration.toString();
assert.sameValue(implicit, "P1Y2M3W4DT5H6M7.98765S", "default precision is auto, and rounding is trunc");

const lambda = duration.toString(() => {});
assert.sameValue(lambda, "P1Y2M3W4DT5H6M7.98765S", "default precision is auto, and rounding is trunc");
