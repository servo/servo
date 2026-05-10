// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When Number is called as part of a new expression it is
    a constructor: it initialises the newly created object
es5id: 15.7.2.1_A1
description: Checking type of the newly created object and it value
---*/
assert.sameValue(typeof new Number(), "object", 'The value of `typeof new Number()` is expected to be "object"');
assert.notSameValue(new Number(), undefined, 'new Number() is expected to not equal ``undefined``');

var x3 = new Number();
assert.sameValue(typeof x3, "object", 'The value of `typeof x3` is expected to be "object"');

var x4 = new Number();
assert.notSameValue(x4, undefined, 'The value of x4 is expected to not equal ``undefined``');
assert.sameValue(typeof new Number(10), "object", 'The value of `typeof new Number(10)` is expected to be "object"');
assert.notSameValue(new Number(10), undefined, 'new Number(10) is expected to not equal ``undefined``');

var x7 = new Number(10);
assert.sameValue(typeof x7, "object", 'The value of `typeof x7` is expected to be "object"');

var x8 = new Number(10);
assert.notSameValue(x8, undefined, 'The value of x8 is expected to not equal ``undefined``');
