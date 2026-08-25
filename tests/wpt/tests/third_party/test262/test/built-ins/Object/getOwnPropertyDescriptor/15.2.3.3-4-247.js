// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-247
description: >
    Object.getOwnPropertyDescriptor - returned value is an instance of
    object
---*/

var obj = {
  "property": 100
};

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert(desc instanceof Object, 'desc instanceof Object !== true');
