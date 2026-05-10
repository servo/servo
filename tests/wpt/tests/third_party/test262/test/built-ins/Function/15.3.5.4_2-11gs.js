// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-11gs
description: >
    Strict mode - checking access to strict function caller from
    strict function (eval used within strict mode)
flags: [onlyStrict]
---*/

assert.throws(TypeError, function() {
  eval("gNonStrict();");
});


function gNonStrict() {
  return gNonStrict.caller;
}
