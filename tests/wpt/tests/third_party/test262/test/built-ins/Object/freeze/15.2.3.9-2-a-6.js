// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-6
description: >
    Object.freeze - 'P' is own accessor property that overrides an
    inherited accessor property
includes: [propertyHelper.js]
---*/

var proto = {};

Object.defineProperty(proto, "foo", {
  get: function() {
    return 0;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();

Object.defineProperty(child, "foo", {
  get: function() {
    return 10;
  },
  configurable: true
});

Object.freeze(child);

verifyProperty(child, "foo", {
  configurable: false,
});

assert.sameValue(child.foo, 10);
