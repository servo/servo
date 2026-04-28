// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is null or undefined, the called function is passed the global
    object as the this value
es5id: 15.3.4.3_A3_T5
description: >
    No any arguments at apply function and it called inside function
    declaration
---*/

function FACTORY() {
  Function("this.feat=\"in da haus\"").apply();
}

var obj = new FACTORY;

assert.sameValue(this["feat"], "in da haus", 'The value of this["feat"] is expected to be "in da haus"');
assert.sameValue(typeof obj.feat, "undefined", 'The value of `typeof obj.feat` is expected to be "undefined"');
