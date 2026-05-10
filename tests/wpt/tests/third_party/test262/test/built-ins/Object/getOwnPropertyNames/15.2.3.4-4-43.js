// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-43
description: >
    Object.getOwnPropertyNames - own accessor property of String
    object 'O' is pushed into the returned array
---*/

var propertyFound = false;

var str = new String("abc");

Object.defineProperty(str, "ownProperty", {
  get: function() {
    return "ownString";
  },
  configurable: true
});

var result = Object.getOwnPropertyNames(str);

for (var p in result) {
  if (result[p] === "ownProperty") {
    propertyFound = true;
    break;
  }
}

assert(propertyFound, 'Property not found');
