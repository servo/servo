// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-5
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a boolean whose
    value is false
---*/

var obj = {
  "false": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, false);

assert.sameValue(desc.value, 1, 'desc.value');
