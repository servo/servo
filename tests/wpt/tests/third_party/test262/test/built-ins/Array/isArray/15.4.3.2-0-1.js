// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-0-1
description: Array.isArray must exist as a function
---*/

var f = Array.isArray;

assert.sameValue(typeof f, "function", 'The value of `typeof f` is expected to be "function"');
