// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-245
description: >
    Object.getOwnPropertyDescriptor - ensure that 'set' property of
    returned object is data property with correct 'enumerable'
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
var accessed = false;

for (var prop in desc) {
  if (prop === "set") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
