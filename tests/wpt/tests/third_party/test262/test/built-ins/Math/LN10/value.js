// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Math.LN10 is a numeric value
esid: sec-math.ln10
info: |
    The Number value for the natural logarithm of 10, which is approximately
    2.302585092994046.

    The precision of this approximation is host-defined.
---*/

assert.sameValue(typeof Math.LN10, 'number');
assert.notSameValue(Math.LN10, NaN);
