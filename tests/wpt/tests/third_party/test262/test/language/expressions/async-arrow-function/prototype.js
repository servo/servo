// Copyright (C) 2024 Linus Groh. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-async-arrow-function-definitions-runtime-semantics-evaluation
description: The prototype of an async arrow function is %AsyncFunction.prototype%.
info: |
    AsyncArrowFunction : async AsyncArrowBindingIdentifier => AsyncConciseBody

    ...
    6. Let closure be OrdinaryFunctionCreate(%AsyncFunction.prototype%, sourceText, parameters,
       AsyncConciseBody, lexical-this, env, privateEnv).
    ...
includes: [wellKnownIntrinsicObjects.js]
---*/

var AsyncFunction = getWellKnownIntrinsicObject('%AsyncFunction%');
assert.sameValue(Object.getPrototypeOf(async () => {}), AsyncFunction.prototype);
