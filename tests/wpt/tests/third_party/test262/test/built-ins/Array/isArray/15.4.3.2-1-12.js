// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-1-12
description: Array.isArray applied to Error object
---*/

assert.sameValue(Array.isArray(new SyntaxError()), false, 'Array.isArray(new SyntaxError()) must return false');
