// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-definitions-evaluation
description: Assignment of function `name` attribute
info: |
    AsyncGeneratorExpression : async function * ( FormalParameters ) { AsyncGeneratorBody }

    1. Let scope be the LexicalEnvironment of the running execution context.
    2. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
       AsyncGeneratorBody, scope, "").
    ...
    6. Return closure.

    AsyncGeneratorExpression : async function * BindingIdentifier ( FormalParameters ) { AsyncGeneratorBody }

    1. Let scope be the running execution context's LexicalEnvironment.
    2. Let funcEnv be ! NewDeclarativeEnvironment(scope).
    3. Let envRec be funcEnv's EnvironmentRecord.
    4. Let name be StringValue of BindingIdentifier.
    5. Perform ! envRec.CreateImmutableBinding(name).
    6. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
       AsyncGeneratorBody, funcEnv, name).
    ...
    11. Return closure.
includes: [propertyHelper.js]
---*/

verifyProperty(async function*() {}, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

verifyProperty(async function* func() {}, "name", {
  value: "func", writable: false, enumerable: false, configurable: true
});
