// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of boolean conversion from number value is false if the argument
    is +0, -0, or NaN; otherwise, is true
esid: sec-toboolean
description: +0, -0 and NaN convert to Boolean by explicit transformation
---*/
assert.sameValue(Boolean(+0), false, 'Boolean(+0) must return false');
assert.sameValue(Boolean(-0), false, 'Boolean(-0) must return false');
assert.sameValue(Boolean(Number.NaN), false, 'Boolean(Number.NaN) must return false');
