// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-243
description: >
    Object.getOwnPropertyDescriptor - ensure that 'set' property of
    returned object is data property with correct 'value' attribute
---*/

var obj = {};
var fun = function() {
  return "ownSetProperty";
};
Object.defineProperty(obj, "property", {
  set: fun,
  configurable: true
});

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert.sameValue(desc.set, fun, 'desc.set');
