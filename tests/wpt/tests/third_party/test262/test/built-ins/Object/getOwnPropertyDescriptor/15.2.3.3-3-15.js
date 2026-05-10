// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-15
description: >
    Object.getOwnPropertyDescriptor applied to a Function object which
    implements its own property get method
---*/

var obj = function(a, b) {
  return a + b;
};
obj[1] = "ownProperty";

var desc = Object.getOwnPropertyDescriptor(obj, "1");

assert.sameValue(desc.value, "ownProperty", 'desc.value');
