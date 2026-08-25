// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-8
description: >
    Object.getOwnPropertyDescriptor - 'P' is own accessor property
    that overrides an inherited accessor property
---*/

var proto = {};
Object.defineProperty(proto, "property", {
  get: function() {
    return "inheritedAccessorProperty";
  },
  configurable: true
});

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
