// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-23gs
description: >
    Strict mode - checking access to strict function caller from
    strict function (Anonymous FunctionExpression defined within a
    FunctionDeclaration inside strict mode)
flags: [onlyStrict]
---*/

function f1() {
  (function() {
    gNonStrict();
  })();
}

assert.throws(TypeError, function() {
  f1();
});

function gNonStrict() {
  return gNonStrict.caller;
}
