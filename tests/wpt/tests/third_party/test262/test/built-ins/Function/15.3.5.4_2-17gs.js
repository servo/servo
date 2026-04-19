// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-17gs
description: >
    Strict mode - checking access to strict function caller from
    strict function (New'ed object from FunctionExpression defined
    within strict mode)
flags: [onlyStrict]
---*/

var f = function() {
  gNonStrict();
}

assert.throws(TypeError, function() {
  new f();
});

function gNonStrict() {
  return gNonStrict.caller;
}
