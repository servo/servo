// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf - non-existent property wouldn't be called
---*/

assert.sameValue([0, , 2].indexOf(undefined), -1, '[0, , 2].indexOf(undefined)');
