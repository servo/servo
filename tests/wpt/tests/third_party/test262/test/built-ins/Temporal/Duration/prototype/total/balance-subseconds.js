// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Balancing from subsecond units to seconds happens correctly
features: [Temporal]
---*/

const pos = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 999, 999999, 999999999);
assert.sameValue(pos.total("seconds"), 2.998998999);

const neg = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -999, -999999, -999999999);
assert.sameValue(neg.total("seconds"), -2.998998999);
