// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%throwtypeerror%
description: >
  %ThrowTypeError% is an anonymous function.
info: |
  %ThrowTypeError% ( )

  9.2.9.1 %ThrowTypeError% ( )
    The %ThrowTypeError% intrinsic is an anonymous built-in function
    object that is defined once for each Realm. The `name` property of a
    %ThrowTypeError% function has the attributes { [[Writable]]: *false*,
    [[Enumerable]]: *false*, [[Configurable]]: *false* }.

  17 ECMAScript Standard Built-in Objects:
    Every built-in function object, including constructors, has a `name`
    property whose value is a String. Functions that are identified as
    anonymous functions use the empty string as the value of the `name`
    property.
    Unless otherwise specified, the `name` property of a built-in function
    object has the attributes { [[Writable]]: *false*, [[Enumerable]]: *false*,
    [[Configurable]]: *true* }.
includes: [propertyHelper.js]
---*/

var ThrowTypeError = Object.getOwnPropertyDescriptor(function() {
  "use strict";
  return arguments;
}(), "callee").get;

verifyProperty(ThrowTypeError, "name", {
  value: "", writable: false, enumerable: false, configurable: false
});
