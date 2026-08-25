// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-2-a-6
description: >
    Object.isFrozen - 'P' is own accessor property that overrides an
    inherited accessor property
---*/

var proto = {};

Object.defineProperty(proto, "foo", {
  get: function() {
    return 12;
  },
  configurable: false
});

var Con = function() {};
Con.prototype = proto;
var child = new Con();


Object.defineProperty(child, "foo", {
  get: function() {
    return 9;
  },
  configurable: true
});

Object.preventExtensions(child);

assert.sameValue(Object.isFrozen(child), false, 'Object.isFrozen(child)');
