// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf - 'length' is own data property on an Array
---*/

var targetObj = {};

Array.prototype[2] = targetObj;


assert.sameValue([0, targetObj].indexOf(targetObj), 1, '[0, targetObj].indexOf(targetObj)');
assert.sameValue([0, 1].indexOf(targetObj), -1, '[0, 1].indexOf(targetObj)');
