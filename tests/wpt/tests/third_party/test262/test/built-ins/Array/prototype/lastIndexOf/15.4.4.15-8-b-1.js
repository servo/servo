// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf - undefined property wouldn't be called
---*/

assert.sameValue([0, , 2].lastIndexOf(undefined), -1, '[0, , 2].lastIndexOf(undefined)');
