// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For newly defined data properties, attributes
    missing from desc should have values set to the defaults from 8.6.1.
es5id: 15.2.3.6-4-2
description: >
    Object.defineProperty sets missing attributes to their default
    values (data properties)(8.12.9 step 4.a.i)
---*/

var o = {};

var desc = {
  value: 1
};
Object.defineProperty(o, "foo", desc);

var propDesc = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(propDesc.value, 1, 'propDesc.value'); // this is the value that was set
assert.sameValue(propDesc.writable, false, 'propDesc.writable'); // false by default
assert.sameValue(propDesc.enumerable, false, 'propDesc.enumerable'); // false by default
assert.sameValue(propDesc.configurable, false, 'propDesc.configurable'); // false by default
