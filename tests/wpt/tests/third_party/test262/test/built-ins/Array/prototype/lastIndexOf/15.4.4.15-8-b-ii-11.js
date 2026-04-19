// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - both array element and search
    element are Objects, and they refer to the same object
---*/

var obj1 = {};
var obj2 = {};
var obj3 = obj2;

assert.sameValue([obj2, obj1].lastIndexOf(obj3), 0, '[obj2, obj1].lastIndexOf(obj3)');
