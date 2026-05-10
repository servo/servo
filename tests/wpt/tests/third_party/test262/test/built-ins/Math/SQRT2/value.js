// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Math.SQRT2 is a numeric value
esid: sec-math.sqrt2
info: |
    The Number value for the square root of 2, which is approximately
    1.4142135623730951.

    The precision of this approximation is host-defined.
---*/

assert.sameValue(typeof Math.SQRT2, 'number');
assert.notSameValue(Math.SQRT2, NaN);
