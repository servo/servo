// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tojson
description: Balancing from subsecond units to seconds happens correctly
features: [Temporal]
---*/

const pos = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 999, 999999, 999999999);
assert.sameValue(pos.toJSON(), "PT2.998998999S");

const neg = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -999, -999999, -999999999);
assert.sameValue(neg.toJSON(), "-PT2.998998999S");
