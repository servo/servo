// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.4.7
description: >
    Completion value when head has no declaration and a "test" expression and
    iteration occurs
info: |
    IterationStatement :
      for ( Expressionopt ; Expressionopt ; Expressionopt ) Statement

    1. If the first Expression is present, then
       a. Let exprRef be the result of evaluating the first Expression.
       b. Let exprValue be GetValue(exprRef).
       c. ReturnIfAbrupt(exprValue).
    2. Return ForBodyEvaluation(the second Expression, the third Expression,
       Statement, « », labelSet).

    13.7.4.8 Runtime Semantics: ForBodyEvaluation
    1. Let V = undefined.
    [...]
    4. Repeat
       a. If test is not [empty], then
          i. Let testRef be the result of evaluating test.
          ii. Let testValue be GetValue(testRef).
          iii. ReturnIfAbrupt(testValue).
          iv. If ToBoolean(testValue) is false, return NormalCompletion(V).
---*/

assert.sameValue(
  eval('var runA; 1; for (runA = true; runA; runA = false) { }'), undefined
);
assert.sameValue(
  eval('var runB; 2; for (runB = true; runB; runB = false) { 3; }'), 3
);
