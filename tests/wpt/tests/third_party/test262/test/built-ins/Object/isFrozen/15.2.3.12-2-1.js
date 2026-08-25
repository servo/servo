// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-2-1
description: >
    Object.isFrozen - inherited data property is not considered into
    the for each loop
---*/

var proto = {};
Object.defineProperty(proto, "Father", {
  value: 10,
  writable: false,
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();

Object.preventExtensions(child);

assert(Object.isFrozen(child), 'Object.isFrozen(child) !== true');
