// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-19gs
description: >
    Strict mode - checking access to strict function caller from
    strict function (New'ed object from Anonymous FunctionExpression
    defined within strict mode)
flags: [onlyStrict]
---*/

assert.throws(TypeError, function() {
  var obj = new(function() {
    gNonStrict();
  });
});

function gNonStrict() {
  return gNonStrict.caller;
}
