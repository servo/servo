// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-1-5
description: Object.isFrozen applies to dense array
---*/

var obj = Object.freeze([0, 1, 2]);

assert(Object.isFrozen(obj), 'Object.isFrozen(obj) !== true');
