// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-237
description: >
    Object.getOwnPropertyDescriptor - ensure that 'configurable'
    property of returned object is data property with correct
    'enumerable' attribute
---*/

var obj = {
  "property": "ownDataProperty"
};

var desc = Object.getOwnPropertyDescriptor(obj, "property");
var accessed = false;

for (var prop in desc) {
  if (prop === "configurable") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
