// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-8-1
description: Function.prototype.bind, type of bound function must be 'function'
---*/

function foo() {}
var o = {};

var bf = foo.bind(o);

assert.sameValue(typeof(bf), 'function', 'typeof(bf)');
