// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-37gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (FunctionExpression defined within an
    Anonymous FunctionExpression with a strict directive prologue)
flags: [noStrict]
---*/

assert.throws(TypeError, function() {
  (function() {
    "use strict";
    var f = function() {
      gNonStrict();
    }
    f();
  })();
});

function gNonStrict() {
  return gNonStrict.caller || gNonStrict.caller.throwTypeError;
}
