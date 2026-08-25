// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-339-2
description: >
    Object.defineProperty - Updating named data property 'P' with
    attributes [[Writable]]: true, [[Enumerable]]: true,
    [[Configurable]]: false to an accessor property does not succeed,
    'O' is an Arguments object (8.12.9 - step 9.a)
---*/

var obj = (function() {
  return arguments;
}());

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

function getFunc() {
  return 20;
}
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "prop", {
    get: getFunc
  });
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");
assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc1.value, 2010, 'desc1.value');
assert.sameValue(obj.prop, 2010, 'obj.prop');
assert.sameValue(typeof desc2.get, "undefined", 'typeof desc2.get');
