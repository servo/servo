// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-51gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (Literal setter includes strict directive
    prologue)
flags: [noStrict]
---*/

var o = {
  set foo(stuff) {
    "use strict";
    gNonStrict();
  }
}

assert.throws(TypeError, function() {
  o.foo = 8;
});

function gNonStrict() {
  return gNonStrict.caller || gNonStrict.caller.throwTypeError;
}
