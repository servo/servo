// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-246
description: >
    Object.getOwnPropertyDescriptor - ensure that 'set' property of
    returned object is data property with correct 'configurable'
    attribute
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

var propDefined = "set" in desc;

delete desc.set;
var propDeleted = "set" in desc;

assert(propDefined, 'propDefined !== true');
assert.sameValue(propDeleted, false, 'propDeleted');
