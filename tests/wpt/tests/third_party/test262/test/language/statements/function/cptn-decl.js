// Copyright (C) 2017 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-definitions-runtime-semantics-evaluation
description: Function declaration completion value is empty.
info: |
    FunctionDeclaration : function BindingIdentifier ( FormalParameters ) { FunctionBody }

    1. Return NormalCompletion(empty).
---*/

assert.sameValue(eval('function f() {}'), undefined);
assert.sameValue(eval('1; function f() {}'), 1);
