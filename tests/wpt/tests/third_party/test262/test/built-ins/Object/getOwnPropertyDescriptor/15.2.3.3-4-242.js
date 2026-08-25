// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-242
description: >
    Object.getOwnPropertyDescriptor - ensure that 'get' property of
    returned object is data property with correct 'configurable'
    attribute
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

var propDefined = "get" in desc;

delete desc.get;
var propDeleted = "get" in desc;

assert(propDefined, 'propDefined !== true');
assert.sameValue(propDeleted, false, 'propDeleted');
