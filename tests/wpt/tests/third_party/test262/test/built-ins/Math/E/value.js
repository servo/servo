// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Math.E is a numeric value
esid: sec-math.e
info: |
    The Number value for e, the base of the natural logarithms, which is
    approximately 2.7182818284590452354.

    The precision of this approximation is host-defined.
---*/

assert.sameValue(typeof Math.E, 'number');
assert.notSameValue(Math.E, NaN);
