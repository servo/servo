// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-1-15
description: Array.isArray applied to the global object
---*/

assert.sameValue(Array.isArray(this), false, 'Array.isArray(this) must return false');
