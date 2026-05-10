// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-16-1
description: Function.prototype.bind, [[Extensible]] of the bound fn is true
---*/

function foo() {}
var o = {};

var bf = foo.bind(o);
var ex = Object.isExtensible(bf);

assert.sameValue(ex, true, 'ex');
