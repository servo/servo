// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Function prototype object is itself a Function object that, when
    invoked, accepts any arguments and returns undefined
es5id: 15.3.4_A2_T3
description: Call Function.prototype(x), where x is undefined variable
---*/

var x;
assert.sameValue(Function.prototype(x), undefined, 'Function.prototype(x) returns undefined');
