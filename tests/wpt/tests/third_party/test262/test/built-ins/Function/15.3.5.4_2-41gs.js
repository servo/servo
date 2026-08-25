// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-41gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (Anonymous FunctionExpression with a strict
    directive prologue defined within a FunctionDeclaration)
flags: [noStrict]
---*/

function f1() {
  return (function() {
    "use strict";
    gNonStrict();
  })();
}

assert.throws(TypeError, function() {
  f1();
});

function gNonStrict() {
  return gNonStrict.caller || gNonStrict.caller.throwTypeError;
}
