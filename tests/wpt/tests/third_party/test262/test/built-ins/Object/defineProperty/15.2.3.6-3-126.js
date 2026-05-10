// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-126
description: >
    Object.defineProperty - 'value' property in 'Attributes' is
    present  (8.10.5 step 5)
---*/

var obj = {};

var attr = {
  value: 100
};

Object.defineProperty(obj, "property", attr);

assert.sameValue(obj.property, 100, 'obj.property');
