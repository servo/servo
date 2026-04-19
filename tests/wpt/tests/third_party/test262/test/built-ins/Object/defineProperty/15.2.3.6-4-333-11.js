// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-333-11
description: >
    ES5 Attributes - indexed property 'P' with attributes
    [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: false
    is writable using simple assignment, 'O' is an Arguments object
---*/

var obj = (function(x) {
  return arguments;
}(1001));

Object.defineProperty(obj, "0", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});
var verifyValue = (obj[0] === 2010);

assert(verifyValue, 'verifyValue !== true');
