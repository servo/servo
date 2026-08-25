// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-40
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is a Boolean Object
    that converts to a string
---*/

var obj = {
  "true": 1
};

var desc = Object.getOwnPropertyDescriptor(obj, new Boolean(true));

assert.sameValue(desc.value, 1, 'desc.value');
