// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-1-3
description: Array.isArray applied to number primitive
---*/

assert.sameValue(Array.isArray(5), false, 'Array.isArray(5) must return false');
