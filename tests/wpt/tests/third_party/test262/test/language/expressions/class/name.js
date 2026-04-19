// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-class-definitions-runtime-semantics-evaluation
description: Assignment of function `name` attribute
info: |
    ClassExpression : class ClassTail

    1. Let value be ? ClassDefinitionEvaluation of ClassTail with arguments
       undefined and "".
    ...
    4. Return value.

    ClassExpression : class BindingIdentifier ClassTail

    1. Let className be StringValue of BindingIdentifier.
    2. Let value be ? ClassDefinitionEvaluation of ClassTail with arguments
       className and className.
    ...
    4. Return value.

    14.6.13 Runtime Semantics: ClassDefinitionEvaluation

    ...
    12. Let constructorInfo be DefineMethod of constructor with arguments proto,
        className as the optional name argument, and constructorParent.
    ...

    14.3.7 Runtime Semantics: DefineMethod

    ...
    7. Let closure be FunctionCreate(kind, UniqueFormalParameters, FunctionBody,
       scope, name, prototype).
    ...

includes: [propertyHelper.js]
---*/

verifyProperty(class {}, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

verifyProperty(class cls {}, "name", {
  value: "cls", writable: false, enumerable: false, configurable: true
});

verifyProperty(class { constructor() {} }, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

verifyProperty(class cls { constructor() {} }, "name", {
  value: "cls", writable: false, enumerable: false, configurable: true
});
