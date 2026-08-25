// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-7
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a number that
    converts to a string (value is NaN)
---*/

var obj = {
  "NaN": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, NaN);

assert.sameValue(desc.value, 1, 'desc.value');
