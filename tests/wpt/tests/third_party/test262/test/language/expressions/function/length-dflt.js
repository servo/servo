// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1.6
description: >
  Default parameters' effect on function length
info: |
  Function length is counted by the non initialized parameters in the left.

  9.2.4 FunctionInitialize (F, kind, ParameterList, Body, Scope)

  [...]
  2. Let len be the ExpectedArgumentCount of ParameterList.
  3. Perform ! DefinePropertyOrThrow(F, "length", PropertyDescriptor{[[Value]]:
     len, [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true}).
  [...]

  FormalsList : FormalParameter

    1. If HasInitializer of FormalParameter is true return 0
    2. Return 1.

  FormalsList : FormalsList , FormalParameter

    1. Let count be the ExpectedArgumentCount of FormalsList.
    2. If HasInitializer of FormalsList is true or HasInitializer of
    FormalParameter is true, return count.
    3. Return count+1.
features: [default-parameters]
includes: [propertyHelper.js]
---*/


var f1 = function (x = 42) {};

verifyProperty(f1, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});

var f2 = function (x = 42, y) {};

verifyProperty(f2, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});

var f3 = function (x, y = 42) {};

verifyProperty(f3, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});

var f4 = function (x, y = 42, z) {};

verifyProperty(f4, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
