// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-2
description: >
    Object.freeze - 'P' is own data property that overrides an
    inherited data property
includes: [propertyHelper.js]
---*/


var proto = {
  foo: 0
}; // default [[Configurable]] attribute value of foo: true

var Con = function() {};
Con.prototype = proto;

var child = new Con();

child.foo = 10; // default [[Configurable]] attribute value of foo: true

Object.freeze(child);

verifyProperty(child, "foo", {
  value: 10,
  writable: false,
  configurable: false,
});
