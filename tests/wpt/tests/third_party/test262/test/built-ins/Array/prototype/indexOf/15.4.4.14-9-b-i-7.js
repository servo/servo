// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is inherited
    data property on an Array
---*/

Array.prototype[0] = true;
Array.prototype[1] = false;
Array.prototype[2] = "true";

assert.sameValue([, , , ].indexOf(true), 0, '[, , , ].indexOf(true)');
assert.sameValue([, , , ].indexOf(false), 1, '[, , , ].indexOf(false)');
assert.sameValue([, , , ].indexOf("true"), 2, '[, , , ].indexOf("true")');
