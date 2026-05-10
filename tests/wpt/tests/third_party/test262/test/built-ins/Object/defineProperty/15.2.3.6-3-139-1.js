// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-139-1
description: >
    Object.defineProperty - 'Attributes' is a Function object which
    implements its own [[Get]] method to access the 'value' property
    of prototype object  (8.10.5 step 5.a)
---*/

var obj = {};

Function.prototype.value = "Function";
var funObj = function(a, b) {
  return a + b;
};

Object.defineProperty(obj, "property", funObj);

assert.sameValue(obj.property, "Function", 'obj.property');
