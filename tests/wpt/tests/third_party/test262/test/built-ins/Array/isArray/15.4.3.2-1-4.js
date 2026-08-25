// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-1-4
description: Array.isArray applied to Number object
---*/

assert.sameValue(Array.isArray(new Number(-3)), false, 'Array.isArray(new Number(-3)) must return false');
