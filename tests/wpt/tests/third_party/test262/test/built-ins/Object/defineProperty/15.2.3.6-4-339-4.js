// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-339-4
description: >
    Object.defineProperty - Updating indexed data property 'P' with
    attributes [[Writable]]: true, [[Enumerable]]: true,
    [[Configurable]]: false to an accessor property does not succeed,
    'O' is an Arguments object (8.12.9 - step 9.a)
---*/

var obj = (function() {
  return arguments;
}());

Object.defineProperty(obj, "0", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});
var propertyDefineCorrect = obj.hasOwnProperty("0");
var desc1 = Object.getOwnPropertyDescriptor(obj, "0");

function getFunc() {
  return 20;
}
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "0", {
    get: getFunc
  });
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "0");
assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc1.value, 2010, 'desc1.value');
assert.sameValue(obj[0], 2010, 'obj[0]');
assert.sameValue(typeof desc2.get, "undefined", 'typeof desc2.get');
