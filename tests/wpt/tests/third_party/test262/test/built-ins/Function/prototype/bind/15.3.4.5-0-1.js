// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-0-1
description: Function.prototype.bind must exist as a function
---*/

var f = Function.prototype.bind;

assert.sameValue(typeof(f), "function", 'typeof(f)');
