// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Math.LOG2E is a numeric value
esid: sec-math.log2e
info: |
    The Number value for the base-2 logarithm of e, the base of the natural
    logarithms; this value is approximately 1.4426950408889634.

    The precision of this approximation is host-defined.
---*/

assert.sameValue(typeof Math.LOG2E, 'number');
assert.notSameValue(Math.LOG2E, NaN);
