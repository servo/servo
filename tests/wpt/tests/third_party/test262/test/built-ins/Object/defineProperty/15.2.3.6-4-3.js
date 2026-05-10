// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For newly defined accessor properties, attributes
    missing from desc should have values set to the defaults from 8.6.1.
es5id: 15.2.3.6-4-3
description: >
    Object.defineProperty sets missing attributes to their default
    values (accessor)(8.12.9 step 4.b.i)
---*/

var o = {};

var getter = function() {
  return 1;
};
var desc = {
  get: getter
};

Object.defineProperty(o, "foo", desc);

var propDesc = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(typeof(propDesc.get), "function", 'typeof(propDesc.get)');
assert.sameValue(propDesc.get, getter, 'propDesc.get'); // the getter must be the function that was provided
assert.sameValue(propDesc.set, undefined, 'propDesc.set'); // undefined by default
assert.sameValue(propDesc.enumerable, false, 'propDesc.enumerable'); // false by default
assert.sameValue(propDesc.configurable, false, 'propDesc.configurable'); // false by default
