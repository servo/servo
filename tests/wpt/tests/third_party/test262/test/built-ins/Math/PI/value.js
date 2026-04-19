// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Math.PI is a numeric value
esid: sec-math.pi
info: |
    The Number value for pi, the ratio of the circumference of a circle to its
    diameter, which is approximately 3.1415926535897932.

    The precision of this approximation is host-defined.
---*/

assert.sameValue(typeof Math.PI, 'number');
assert.notSameValue(Math.PI, NaN);
