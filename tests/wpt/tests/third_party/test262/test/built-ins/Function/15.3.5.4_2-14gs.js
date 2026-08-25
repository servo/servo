// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-14gs
description: >
    Strict mode - checking access to non-strict function caller from
    non-strict function (indirect eval includes strict directive
    prologue)
flags: [noStrict]
features: [caller]
---*/

var my_eval = eval;
my_eval("\"use strict\";\ngNonStrict();");


function gNonStrict() {
  return gNonStrict.caller;
}
