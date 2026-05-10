// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-360-6
description: >
    ES5 Attributes - Updating indexed data property 'P' whose
    attributes are [[Writable]]: false, [[Enumerable]]: true,
    [[Configurable]]: true to an accessor property, 'O' is an
    Arguments object (8.12.9 - step 9.b.i)
---*/

var obj = (function() {
  return arguments;
}());

Object.defineProperty(obj, "0", {
  value: 2010,
  writable: false,
  enumerable: true,
  configurable: true
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "0");

function getFunc() {
  return 20;
}
Object.defineProperty(obj, "0", {
  get: getFunc
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "0");

assert(desc1.hasOwnProperty("value"), 'desc1.hasOwnProperty("value") !== true');
assert(desc2.hasOwnProperty("get"), 'desc2.hasOwnProperty("get") !== true');
assert.sameValue(desc2.enumerable, true, 'desc2.enumerable');
assert.sameValue(desc2.configurable, true, 'desc2.configurable');
assert.sameValue(obj[0], 20, 'obj[0]');
assert.sameValue(typeof desc2.set, "undefined", 'typeof desc2.set');
assert.sameValue(desc2.get, getFunc, 'desc2.get');
