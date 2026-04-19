// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-32
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is applied to an
    empty string
---*/

var obj = {
  "": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, "");

assert.sameValue(desc.value, 1, 'desc.value');
