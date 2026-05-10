// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.sign
description: Basic functionality
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
assert.sameValue(instance.sign, 1);

const smallInstance = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 1);
assert.sameValue(smallInstance.sign, 1);

const negInstance = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
assert.sameValue(negInstance.sign, -1);

const smallNegInstance = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, -1);
assert.sameValue(smallNegInstance.sign, -1);
