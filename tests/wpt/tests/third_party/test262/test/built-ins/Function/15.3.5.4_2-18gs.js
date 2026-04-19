// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-18gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (New'ed object from FunctionExpression
    includes strict directive prologue)
flags: [noStrict]
---*/

var f = function() {
  "use strict";
  gNonStrict();
}

assert.throws(TypeError, function() {
  new f();
});

function gNonStrict() {
  return gNonStrict.caller || gNonStrict.caller.throwTypeError;
}
