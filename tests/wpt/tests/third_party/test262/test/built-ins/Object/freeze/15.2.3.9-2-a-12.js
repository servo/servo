// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-12
description: >
    Object.freeze - 'P' is own index property of a String object that
    implements its own [[GetOwnProperty]]
includes: [propertyHelper.js]
---*/


// default [[Configurable]] attribute value of "0": true
var strObj = new String("abc");

Object.freeze(strObj);

verifyProperty(strObj, "0", {
  value: "a",
  writable: false,
  configurable: false,
});
