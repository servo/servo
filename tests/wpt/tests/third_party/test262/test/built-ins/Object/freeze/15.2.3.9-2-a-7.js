// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-7
description: >
    Object.freeze - 'P' is own named property of an Arguments object
    that implements its own [[GetOwnProperty]]
includes: [propertyHelper.js]
---*/

var argObj = (function() {
  return arguments;
}());

argObj.foo = 10; // default [[Configurable]] attribute value of foo: true

Object.freeze(argObj);

verifyProperty(argObj, "foo", {
  value: 10,
  writable: false,
  configurable: false,
});
