// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.3.6
description: >
    Completion value when iteration completes due to expression value
info: |
    IterationStatement : while ( Expression ) Statement

    1. Let V = undefined.
    2. Repeat
       a. Let exprRef be the result of evaluating Expression.
       b. Let exprValue be GetValue(exprRef).
       c. ReturnIfAbrupt(exprValue).
       d. If ToBoolean(exprValue) is false, return NormalCompletion(V).
       e. Let stmt be the result of evaluating Statement.
       f. If LoopContinues (stmt, labelSet) is false, return
          Completion(UpdateEmpty(stmt, V)).
       g. If stmt.[[value]] is not empty, let V = stmt.[[value]].
---*/

assert.sameValue(eval('var count1 = 2; 1; while (count1 -= 1) { }'), undefined);
assert.sameValue(eval('var count2 = 2; 2; while (count2 -= 1) { 3; }'), 3);
