// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tojson
description: Temporal.Duration.toJSON handles negative components
features: [Temporal]
---*/
const d = new Temporal.Duration(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
const expected = "-P1Y1M1W1DT1H1M1.001001001S";
assert.sameValue(d.toJSON(), expected, "toJSON with negative components");
