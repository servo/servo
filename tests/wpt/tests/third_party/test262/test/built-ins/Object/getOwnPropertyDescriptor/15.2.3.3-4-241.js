// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-241
description: >
    Object.getOwnPropertyDescriptor - ensure that 'get' property of
    returned object is data property with correct 'enumerable'
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
var accessed = false;

for (var prop in desc) {
  if (prop === "get") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
