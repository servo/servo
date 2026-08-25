// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-7
description: >
    Object.getOwnPropertyDescriptor - 'P' is own accessor property
    that overrides an inherited data property
---*/

var proto = {
  property: "inheritedDataProperty"
};

var Con = function() {};
Con.ptototype = proto;

var child = new Con();
var fun = function() {
  return "ownAccessorProperty";
};
Object.defineProperty(child, "property", {
  get: fun,
  configurable: true
});

var desc = Object.getOwnPropertyDescriptor(child, "property");

assert.sameValue(desc.get, fun, 'desc.get');
