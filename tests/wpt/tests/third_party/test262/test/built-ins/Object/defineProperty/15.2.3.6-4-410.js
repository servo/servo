// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-410
description: >
    ES5 Attributes - Failed to add a property to an object when the
    object's prototype has a property with the same name and
    [[Writable]] set to false (JSON)
includes: [propertyHelper.js]
---*/

Object.defineProperty(Object.prototype, "prop", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: true
});

assert(!JSON.hasOwnProperty("prop"));
verifyNotWritable(JSON, "prop", "noCheckOwnProp");
assert.sameValue(JSON.prop, 1001);
