// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of boolean conversion from number value is false if the argument
    is +0, -0, or NaN; otherwise, is true
esid: sec-toboolean
description: >
    Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY,
    Number.MAX_VALUE, Number.MIN_VALUE and some numbers convert to
    Boolean by explicit transformation
---*/
assert.sameValue(Boolean(Number.POSITIVE_INFINITY), true, 'Boolean(Number.POSITIVE_INFINITY) must return true');
assert.sameValue(Boolean(Number.NEGATIVE_INFINITY), true, 'Boolean(Number.NEGATIVE_INFINITY) must return true');
assert.sameValue(Boolean(Number.MAX_VALUE), true, 'Boolean(Number.MAX_VALUE) must return true');
assert.sameValue(Boolean(Number.MIN_VALUE), true, 'Boolean(Number.MIN_VALUE) must return true');
assert.sameValue(Boolean(13), true, 'Boolean(13) must return true');
assert.sameValue(Boolean(-13), true, 'Boolean(-13) must return true');
assert.sameValue(Boolean(1.3), true, 'Boolean(1.3) must return true');
assert.sameValue(Boolean(-1.3), true, 'Boolean(-1.3) must return true');
