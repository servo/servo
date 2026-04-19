// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-42
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is an object which
    has an own toString method
---*/

var obj = {
  "abc": 1
};

var ownProp = {
  toString: function() {
    return "abc";
  }
};

var desc = Object.getOwnPropertyDescriptor(obj, ownProp);

assert.sameValue(desc.value, 1, 'desc.value');
