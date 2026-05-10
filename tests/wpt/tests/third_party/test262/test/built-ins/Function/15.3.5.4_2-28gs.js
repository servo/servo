// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-28gs
description: >
    Strict mode - checking access to strict function caller from
    strict function (FunctionExpression defined within an Anonymous
    FunctionExpression inside strict mode)
flags: [onlyStrict]
---*/

assert.throws(TypeError, function() {
  (function() {
    var f = function() {
      gNonStrict();
    }
    f();
  })();
});


function gNonStrict() {
  return gNonStrict.caller;
}
