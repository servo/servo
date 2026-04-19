// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-10
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a number that
    converts to a string (value is -0)
---*/

var obj = {
  "0": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, -0);

assert.sameValue(desc.value, 1, 'desc.value');
