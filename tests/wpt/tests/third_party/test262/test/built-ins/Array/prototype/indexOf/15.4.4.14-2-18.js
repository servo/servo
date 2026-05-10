// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf applied to String object, which implements
    its own property get method
---*/

var str = new String("012");

String.prototype[3] = "3";

assert.sameValue(Array.prototype.indexOf.call(str, "2"), 2, 'Array.prototype.indexOf.call(str, "2")');
assert.sameValue(Array.prototype.indexOf.call(str, "3"), -1, 'Array.prototype.indexOf.call(str, "3")');
