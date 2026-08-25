// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-97gs
description: >
    Strict mode - checking access to strict function caller from bound
    non-strict function (FunctionDeclaration includes strict directive
    prologue)
flags: [noStrict]
---*/

var gNonStrict = gNonStrictBindee.bind(null);

function f() {
  "use strict";
  gNonStrict();
}

assert.throws(TypeError, function() {
  f();
});

function gNonStrictBindee() {
  return gNonStrictBindee.caller || gNonStrictBindee.caller.throwTypeError;
}
