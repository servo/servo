// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-0-3
description: Array.isArray return true if its argument is an Array
---*/

assert.sameValue(Array.isArray([]), true, 'Array.isArray([]) must return true');
