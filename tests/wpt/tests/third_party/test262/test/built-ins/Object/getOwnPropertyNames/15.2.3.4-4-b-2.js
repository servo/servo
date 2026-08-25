// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-b-2
description: >
  Object.getOwnPropertyNames - all own properties are pushed into
  the returned array
includes: [compareArray.js]
---*/

var obj = {
  "a": "a"
};

Object.defineProperty(obj, "b", {
  get: function() {
    return "b";
  },
  enumerable: false,
  configurable: true
});

Object.defineProperty(obj, "c", {
  get: function() {
    return "c";
  },
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "d", {
  value: "d",
  enumerable: false,
  configurable: true
});

assert.compareArray(Object.getOwnPropertyNames(obj), ["a", "b", "c", "d"]);
