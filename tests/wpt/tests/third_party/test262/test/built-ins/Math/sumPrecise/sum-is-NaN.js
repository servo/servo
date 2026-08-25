// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.sumprecise
description: Math.sumPrecise returns NaN when input has NaN or when adding infinities
features: [Math.sumPrecise]
---*/

assert.sameValue(Math.sumPrecise([NaN]), NaN);
assert.sameValue(Math.sumPrecise([Infinity, -Infinity]), NaN);
assert.sameValue(Math.sumPrecise([-Infinity, Infinity]), NaN);
