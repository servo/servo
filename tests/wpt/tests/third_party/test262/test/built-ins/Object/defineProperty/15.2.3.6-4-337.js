// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-337
description: >
    ES5 Attributes - fail to update [[Enumerable]] attribute of data
    property ([[Writable]] is true, [[Enumerable]] is true,
    [[Configurable]] is false) to different value
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});

var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "prop", {
    enumerable: false
  });
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");
assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc1.enumerable, true, 'desc1.enumerable');
assert.sameValue(obj.prop, 2010, 'obj.prop');
assert.sameValue(desc2.enumerable, true, 'desc2.enumerable');
