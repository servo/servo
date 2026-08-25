// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf when 'length' is own data property
    that overrides an inherited data property on an Array
---*/

var targetObj = {};
var arrProtoLen;

arrProtoLen = Array.prototype.length;
Array.prototype.length = 0;

assert.sameValue([0, targetObj, 2].lastIndexOf(targetObj), 1, '[0, targetObj, 2].lastIndexOf(targetObj)');
