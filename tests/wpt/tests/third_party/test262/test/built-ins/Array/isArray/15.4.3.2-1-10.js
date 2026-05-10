// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-1-10
description: Array.isArray applied to RegExp object
---*/

assert.sameValue(Array.isArray(new RegExp()), false, 'Array.isArray(new RegExp()) must return false');
