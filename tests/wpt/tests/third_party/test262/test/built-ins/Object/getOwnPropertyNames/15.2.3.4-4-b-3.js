// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-b-3
description: >
    Object.getOwnPropertyNames - own property named empty('') is
    pushed into the returned array
---*/

var propertyFound = false;

var obj = {
  "": "empty"
};

var result = Object.getOwnPropertyNames(obj);

for (var p in result) {
  if (result[p] === "") {
    propertyFound = true;
    break;
  }
}

assert(propertyFound, 'Property not found');
