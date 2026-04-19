// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-25
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a number that
    converts to a string (value is 1e-7)
---*/

var obj = {
  "1e-7": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, 1e-7);

assert.sameValue(desc.value, 1, 'desc.value');
