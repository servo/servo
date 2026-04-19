// Copyright (C) 2017 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-async-function-definitions-runtime-semantics-evaluation
description: Async function declaration completion value is empty.
info: |
    AsyncFunctionDeclaration : async [no LineTerminator here] function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }

    1. Return NormalCompletion(empty).
---*/

assert.sameValue(eval('async function f() {}'), undefined);
assert.sameValue(eval('1; async function f() {}'), 1);
