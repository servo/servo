// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-222
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    property, the [[Set]] field of 'desc' and the [[Set]] attribute
    value of 'P' are two objects which refer to the same object
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

function set_func(value) {
  arr.setVerifyHelpProp = value;
}

Object.defineProperty(arr, "0", {
  set: set_func
});

Object.defineProperties(arr, {
  "0": {
    set: set_func
  }
});
verifyWritable(arr, "0", "setVerifyHelpProp");

verifyProperty(arr, "0", {
  enumerable: false,
  configurable: false,
});
