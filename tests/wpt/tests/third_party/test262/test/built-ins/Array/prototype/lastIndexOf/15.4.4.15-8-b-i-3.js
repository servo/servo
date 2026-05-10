// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - element to be retrieved is own data
    property that overrides an inherited data property on an Array
---*/

Array.prototype[0] = Object;

assert.sameValue([Object.prototype].lastIndexOf(Object.prototype), 0, '[Object.prototype].lastIndexOf(Object.prototype)');
