// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-165
description: >
    Object.create - one property in 'Properties' is a Function object
    which implements its own [[Get]] method to access the 'value'
    property (8.10.5 step 5.a)
---*/

var Func = function(a, b) {
  return a + b;
};

var fun = new Func();
fun.value = "FunValue";

var newObj = Object.create({}, {
  prop: fun
});

assert.sameValue(newObj.prop, "FunValue", 'newObj.prop');
