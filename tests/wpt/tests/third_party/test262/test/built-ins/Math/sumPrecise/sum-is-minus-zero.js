// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.sumprecise
description: Math.sumPrecise returns -0 on an empty list or list of all -0
features: [Math.sumPrecise]
---*/

assert.sameValue(Math.sumPrecise([]), -0);
assert.sameValue(Math.sumPrecise([-0]), -0);
assert.sameValue(Math.sumPrecise([-0, -0]), -0);
assert.sameValue(Math.sumPrecise([-0, 0]), 0);
