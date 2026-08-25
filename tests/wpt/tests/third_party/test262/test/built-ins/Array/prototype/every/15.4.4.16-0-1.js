// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every must exist as a function
---*/

var f = Array.prototype.every;

assert.sameValue(typeof(f), "function", 'typeof(f)');
