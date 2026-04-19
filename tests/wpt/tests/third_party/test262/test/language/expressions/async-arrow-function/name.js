// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-async-arrow-function-definitions-runtime-semantics-evaluation
description: Assignment of function `name` attribute
info: |
    AsyncArrowFunction : async AsyncArrowBindingIdentifier => AsyncConciseBody

    1. Let scope be the LexicalEnvironment of the running execution context.
    2. Let parameters be AsyncArrowBindingIdentifier.
    3. Let closure be ! AsyncFunctionCreate(Arrow, parameters, AsyncConciseBody,
       scope, "").
    ...
    5. Return closure.

    AsyncArrowFunction : CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody

    1. Let scope be the LexicalEnvironment of the running execution context.
    2. Let head be CoveredAsyncArrowHead of CoverCallExpressionAndAsyncArrowHead.
    3. Let parameters be the ArrowFormalParameters of head.
    4. Let closure be ! AsyncFunctionCreate(Arrow, parameters, AsyncConciseBody,
       scope, "").
    ...
    6. Return closure. 
includes: [propertyHelper.js]
---*/

verifyProperty(async x => {}, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

verifyProperty(async () => {}, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});
