// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-36
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is applied to
    string '123���¦�cd'
---*/

var obj = {
  "123���¦�cd": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, "123���¦�cd");

assert.sameValue(desc.value, 1, 'desc.value');
