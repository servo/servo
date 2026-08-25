// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf applied to Date object
---*/

var obj = new Date(0);
obj.length = 2;
obj[1] = true;

assert.sameValue(Array.prototype.indexOf.call(obj, true), 1, 'Array.prototype.indexOf.call(obj, true)');
