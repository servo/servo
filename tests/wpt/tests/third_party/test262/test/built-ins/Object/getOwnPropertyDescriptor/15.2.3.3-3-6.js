// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-6
description: >
    Object.getOwnPropertyDescriptor - 'P' is inherited accessor
    property
---*/

var proto = {};
var fun = function() {
  return "ownAccessorProperty";
};
Object.defineProperty(proto, "property", {
  get: fun,
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();

var desc = Object.getOwnPropertyDescriptor(child, "property");

assert.sameValue(typeof desc, "undefined", 'typeof desc');
