// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.4.7
description: >
    Completion value when head has a declaration and a "test" expression and iteration occurs
info: |
    IterationStatement :
      for ( var VariableDeclarationList ; Expressionopt ; Expressionopt ) Statement

    1. Let varDcl be the result of evaluating VariableDeclarationList.
    2. ReturnIfAbrupt(varDcl).
    3. Return ForBodyEvaluation(the first Expression, the second Expression,
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
  eval('1; for (var runA = true; runA; runA = false) { }'), undefined
);
assert.sameValue(
  eval('2; for (var runB = true; runB; runB = false) { 3; }'), 3
);
