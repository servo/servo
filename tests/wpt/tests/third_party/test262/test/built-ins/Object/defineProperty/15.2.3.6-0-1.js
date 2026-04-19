// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-0-1
description: Object.defineProperty must exist as a function
---*/

var f = Object.defineProperty;

assert.sameValue(typeof(f), "function", 'typeof(f)');
