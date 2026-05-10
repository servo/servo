// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-5
description: >
    Object.freeze - 'P' is own accessor property that overrides an
    inherited data property
includes: [propertyHelper.js]
---*/


var proto = {};

proto.foo = 0; // default [[Configurable]] attribute value of foo: true

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
