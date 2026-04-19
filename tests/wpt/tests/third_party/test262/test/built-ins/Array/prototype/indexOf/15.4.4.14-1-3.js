// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf applied to boolean primitive
---*/

var targetObj = {};

Boolean.prototype[1] = targetObj;
Boolean.prototype.length = 2;

assert.sameValue(Array.prototype.indexOf.call(true, targetObj), 1, 'Array.prototype.indexOf.call(true, targetObj)');
