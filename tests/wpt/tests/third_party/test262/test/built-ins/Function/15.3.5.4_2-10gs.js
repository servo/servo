// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-10gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (New'ed Function constructor includes strict
    directive prologue)
flags: [noStrict]
---*/

var f = new Function("\"use strict\";\ngNonStrict();");

assert.throws(TypeError, function() {
  f();
});

function gNonStrict() {
  return gNonStrict.caller || gNonStrict.caller.throwTypeError;
}
