// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-239
description: >
    Object.getOwnPropertyDescriptor - ensure that 'get' property of
    returned object is data property with correct 'value' attribute
---*/

var obj = {};
var fun = function() {
  return "ownDataProperty";
};
Object.defineProperty(obj, "property", {
  get: fun,
  configurable: true
});

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert.sameValue(desc.get, fun, 'desc.get');
