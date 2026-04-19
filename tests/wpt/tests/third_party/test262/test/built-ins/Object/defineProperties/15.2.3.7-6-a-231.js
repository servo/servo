// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-231
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    property,  'P' is data property and 'desc' is accessor descriptor,
    and the [[Configurable]] attribute value of 'P' is true, test 'P'
    is converted from data property to accessor property  (15.4.5.1
    step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];
arr[1] = 3; // default value of attributes: writable: true, configurable: true, enumerable: true

function set_fun(value) {
  arr.setVerifyHelpProp = value;
}

Object.defineProperties(arr, {
  "1": {
    set: set_fun
  }
});

verifyWritable(arr, "1", "setVerifyHelpProp");

verifyProperty(arr, "1", {
  enumerable: true,
  configurable: true,
});
