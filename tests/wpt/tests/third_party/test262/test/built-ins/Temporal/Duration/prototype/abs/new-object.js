// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.abs
description: Temporal.Duration.prototype.abs returns a new object.
features: [Temporal]
---*/

let d1 = new Temporal.Duration();
assert.notSameValue(d1.abs(), d1);

let d2 = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
assert.notSameValue(d2.abs(), d2);

let d3 = new Temporal.Duration(1e5, 2e5, 3e5, 4e5, 5e5, 6e5, 7e5, 8e5, 9e5, 10e5);
assert.notSameValue(d3.abs(), d3);

let d4 = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
assert.notSameValue(d4.abs(), d4);

// Test with some zeros
let d5 = new Temporal.Duration(1, 0, 3, 0, 5, 0, 7, 0, 9, 0);
assert.notSameValue(d5.abs(), d5);

let d6 = new Temporal.Duration(0, 2, 0, 4, 0, 6, 0, 8, 0, 10);
assert.notSameValue(d6.abs(), d6);
