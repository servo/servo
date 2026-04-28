// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-7gs
description: >
    Strict mode - checking access to non-strict function caller from
    strict function (Function constructor defined within strict mode)
flags: [onlyStrict]
---*/

var f = Function("return gNonStrict();");

assert.throws(TypeError, function() {
  f();
});

function gNonStrict() {
  return gNonStrict.caller;
}
