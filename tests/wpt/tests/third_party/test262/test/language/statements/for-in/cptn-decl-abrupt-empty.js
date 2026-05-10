// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.5.11
description: >
    Completion value when head has a declaration and iteration is cancelled
info: |
    IterationStatement : for ( var ForBinding in Expression ) Statement

    1. Let keyResult be ForIn/OfHeadEvaluation( « », Expression, enumerate).
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
       m. If LoopContinues(result, labelSet) is false, return
          IteratorClose(iterator, UpdateEmpty(result, V)).
---*/

assert.sameValue(eval('1; for (var a in { x: 0 }) { break; }'), undefined);
assert.sameValue(eval('2; for (var b in { x: 0 }) { 3; break; }'), 3);

assert.sameValue(
  eval('4; outer: do { for (var a in { x: 0 }) { continue outer; } } while (false)'),
  undefined
);
assert.sameValue(
  eval('5; outer: do { for (var b in { x: 0 }) { 6; continue outer; } } while (false)'),
  6
);
