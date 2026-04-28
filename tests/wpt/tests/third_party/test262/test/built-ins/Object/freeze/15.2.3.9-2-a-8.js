// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-8
description: >
    Object.freeze - 'P' is own named property of the String object
    that implements its own [[GetOwnProperty]]
includes: [propertyHelper.js]
---*/

var strObj = new String("abc");

strObj.foo = 10; // default [[Configurable]] attribute value of foo: true

Object.freeze(strObj);

verifyProperty(strObj, "foo", {
  value: 10,
  writable: false,
  configurable: false,
});
