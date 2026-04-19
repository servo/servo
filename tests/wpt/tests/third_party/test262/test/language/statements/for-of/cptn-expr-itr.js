// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.5.11
description: >
    Completion value when head has no declaration and iteration occurs
info: |
    IterationStatement :
        for ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ForIn/OfHeadEvaluation( « »,
       AssignmentExpression, iterate).
    2. ReturnIfAbrupt(keyResult).
    3. Return ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

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

assert.sameValue(eval('var a; 1; for (a of [0]) { }'), undefined);
assert.sameValue(eval('var b; 2; for (b of [0]) { 3; }'), 3);
