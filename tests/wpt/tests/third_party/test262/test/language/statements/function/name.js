// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 14.1.19
description: Assignment of function `name` attribute
info: |
    FunctionDeclaration :
        function BindingIdentifier ( FormalParameters ) { FunctionBody }

    1. Let name be StringValue of BindingIdentifier.
    2. Let F be FunctionCreate(Normal, FormalParameters, FunctionBody, scope, strict).
    3. Perform MakeConstructor(F).
    4. Perform SetFunctionName(F, name).
    5. Return F.
includes: [propertyHelper.js]
---*/

function func() {}

verifyProperty(func, "name", {
  value: "func",
  writable: false,
  enumerable: false,
  configurable: true,
});
