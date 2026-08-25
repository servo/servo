// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Math.LOG10E is a numeric value
esid: sec-math.log10e
info: |
    The Number value for the base-10 logarithm of e, the base of the natural
    logarithms; this value is approximately 0.4342944819032518.

    The precision of this approximation is host-defined.
---*/

assert.sameValue(typeof Math.LOG10E, 'number');
assert.notSameValue(Math.LOG10E, NaN);
