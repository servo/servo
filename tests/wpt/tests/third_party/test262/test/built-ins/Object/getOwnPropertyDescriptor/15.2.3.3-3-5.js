// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-5
description: Object.getOwnPropertyDescriptor - 'P' is own accessor property
---*/

var obj = {};
var fun = function() {
  return "ownAccessorProperty";
};
Object.defineProperty(obj, "property", {
  get: fun,
  configurable: true
});

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert.sameValue(desc.get, fun, 'desc.get');
