// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-11
description: >
    Object.freeze - 'P' is own index property of the Arguments object
    that implements its own [[GetOwnProperty]]
includes: [propertyHelper.js]
---*/


// default [[Configurable]] attribute value of "0": true
var argObj = (function() {
  return arguments;
}(1, 2, 3));

Object.freeze(argObj);

verifyProperty(argObj, "0", {
  value: 1,
  writable: false,
  configurable: false,
});
