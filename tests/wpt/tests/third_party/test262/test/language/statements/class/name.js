// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 14.5.15
description: Assignment of function `name` attribute
info: |
    ClassDeclaration : class BindingIdentifier ClassTail

    [...]
    6. If hasNameProperty is false, then perform SetFunctionName(value,
       className).
includes: [propertyHelper.js]
---*/

class Test262 {}

verifyProperty(Test262, "name", {
  value: "Test262",
  writable: false,
  enumerable: false,
  configurable: true,
});
