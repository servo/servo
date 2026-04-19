// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    15.3.4.5 step 2 specifies that a TypeError must be thrown if the Target
    is not callable.
es5id: 15.3.4.5-2-1
description: >
    Function.prototype.bind throws TypeError if the Target is not
    callable (but an instance of Function)
---*/

foo.prototype = Function.prototype;
// dummy function
function foo() {}
var f = new foo();
assert.throws(TypeError, function() {
  f.bind();
});
