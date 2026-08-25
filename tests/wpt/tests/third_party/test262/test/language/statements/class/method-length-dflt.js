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

class C1 { m(x = 42) {} }

var m1 = C1.prototype.m;

verifyProperty(m1, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});

class C2 { m(x = 42, y) {} }

var m2 = C2.prototype.m;

verifyProperty(m2, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});

class C3 { m(x, y = 42) {} }

var m3 = C3.prototype.m;

verifyProperty(m3, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});

class C4 { m(x, y = 42, z) {} }

var m4 = C4.prototype.m;

verifyProperty(m4, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
