// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.11.7
description: Statement completion value when body returns a normal completion
info: |
    WithStatement : with ( Expression ) Statement

    [...]
    8. Let C be the result of evaluating Statement.
    9. Set the running execution context’s Lexical Environment to oldEnv.
    10. If C.[[type]] is normal and C.[[value]] is empty, return
        NormalCompletion(undefined).
    11. Return Completion(C).
flags: [noStrict]
---*/

assert.sameValue(eval('1; with({}) { }'), undefined);
assert.sameValue(eval('2; with({}) { 3; }'), 3);
