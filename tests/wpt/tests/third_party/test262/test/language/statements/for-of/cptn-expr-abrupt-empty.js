// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.5.11
description: >
    Completion value when head has no declaration and iteration is cancelled
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
       m. If LoopContinues(result, labelSet) is false, return
          IteratorClose(iterator, UpdateEmpty(result, V)).
---*/

assert.sameValue(eval('var a; 1; for (a of [0]) { break; }'), undefined);
assert.sameValue(eval('var b; 2; for (b of [0]) { 3; break; }'), 3);

assert.sameValue(
  eval('var a; 4; outer: do { for (a of [0]) { continue outer; } } while (false)'),
  undefined
);
assert.sameValue(
  eval('var b; 5; outer: do { for (b of [0]) { 6; continue outer; } } while (false)'),
  6
);
