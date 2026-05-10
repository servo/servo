// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-15
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a number that
    converts to a string (value is -Infinity)
---*/

var obj = {
  "-Infinity": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, -Infinity);

assert.sameValue(desc.value, 1, 'desc.value');
