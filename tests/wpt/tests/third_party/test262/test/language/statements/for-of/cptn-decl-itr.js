// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.5.11
description: >
    Completion value when head has a declaration and iteration occurs
info: |
    IterationStatement : for ( var ForBinding of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ForIn/OfHeadEvaluation( « »,
       AssignmentExpression, iterate).
    2. ReturnIfAbrupt(keyResult).
    3. Return ForIn/OfBodyEvaluation(ForBinding, Statement, keyResult,
       varBinding, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    2. Let V = undefined.
    [...]
    5. Repeat
       a. Let nextResult be IteratorStep(iterator).
       b. ReturnIfAbrupt(nextResult).
       c. If nextResult is false, return NormalCompletion(V).
       [...]
       k. Let result be the result of evaluating stmt.
       [...]
       n. If result.[[value]] is not empty, let V be result.[[value]].
---*/

assert.sameValue(eval('1; for (var a of [0]) { }'), undefined);
assert.sameValue(eval('2; for (var b of [0]) { 3; }'), 3);
