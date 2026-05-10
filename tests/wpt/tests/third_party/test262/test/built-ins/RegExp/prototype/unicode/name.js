// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.15
description: >
    "name" property of `unicode` accessor function
info: |
    The value of the name property of this function is "get ".

    ES6 section 17:

    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value is a
    String.

    [...]

    Functions that are specified as get or set accessor functions of built-in
    properties have "get " or "set " prepended to the property name string.

    [...]

    Unless otherwise specified, the name property of a built-in
    Function object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var getter = Object.getOwnPropertyDescriptor(RegExp.prototype, 'unicode').get;

verifyProperty(getter, "name", {
  value: "get unicode",
  writable: false,
  enumerable: false,
  configurable: true
});
