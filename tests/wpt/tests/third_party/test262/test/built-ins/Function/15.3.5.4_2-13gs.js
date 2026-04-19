// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-13gs
description: >
    Strict mode - checking access to non-strict function caller from
    strict function (indirect eval used within strict mode)
flags: [onlyStrict]
---*/

var my_eval = eval;

assert.throws(TypeError, function() {
  my_eval("gNonStrict();");
});

function gNonStrict() {
  return gNonStrict.caller;
}
