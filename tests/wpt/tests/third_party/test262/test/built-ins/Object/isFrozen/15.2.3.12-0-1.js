// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-0-1
description: Object.isFrozen must exist as a function
---*/

var f = Object.isFrozen;

assert.sameValue(typeof(f), "function", 'typeof(f)');
