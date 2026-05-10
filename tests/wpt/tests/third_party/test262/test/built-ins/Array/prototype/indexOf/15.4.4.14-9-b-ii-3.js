// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - both type of array element and type of
    search element are null
---*/

assert.sameValue([null].indexOf(null), 0, '[null].indexOf(null)');
