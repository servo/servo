// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%throwtypeerror%
description: >
  %ThrowTypeError% throws a TypeError when called.
info: |
  %ThrowTypeError% ( )

  When %ThrowTypeError% is called it performs the following steps:

    1. Throw a TypeError exception.
---*/

var ThrowTypeError = Object.getOwnPropertyDescriptor(function() {
  "use strict";
  return arguments;
}(), "callee").get;

assert.throws(TypeError, function() {
  ThrowTypeError();
});
