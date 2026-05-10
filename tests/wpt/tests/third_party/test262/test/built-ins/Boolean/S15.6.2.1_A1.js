// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When Boolean is called as part of a new expression it is
    a constructor: it initialises the newly created object
esid: sec-boolean-constructor
description: Checking type of the newly created object and it value
---*/

assert.sameValue(typeof new Boolean(), "object", 'The value of `typeof new Boolean()` is expected to be "object"');
assert.notSameValue(new Boolean(), undefined, 'new Boolean() is expected to not equal ``undefined``');

var x3 = new Boolean();
assert.sameValue(typeof x3, "object", 'The value of `typeof x3` is expected to be "object"');

var x4 = new Boolean();
assert.notSameValue(x4, undefined, 'The value of x4 is expected to not equal ``undefined``');
assert.sameValue(typeof new Boolean(1), "object", 'The value of `typeof new Boolean(1)` is expected to be "object"');
assert.notSameValue(new Boolean(1), undefined, 'new Boolean(1) is expected to not equal ``undefined``');

var x7 = new Boolean(1);
assert.sameValue(typeof x7, "object", 'The value of `typeof x7` is expected to be "object"');

var x8 = new Boolean(1);
assert.notSameValue(x8, undefined, 'The value of x8 is expected to not equal ``undefined``');
