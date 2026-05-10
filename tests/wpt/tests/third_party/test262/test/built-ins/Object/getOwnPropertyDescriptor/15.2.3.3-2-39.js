// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-39
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a String Object
    that converts to a string
---*/

var obj = {
  "Hello": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, new String("Hello"));

assert.sameValue(desc.value, 1, 'desc.value');
