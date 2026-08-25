// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.3.6
description: >
    Completion value when iteration completes due to an empty abrupt completion
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
---*/

assert.sameValue(eval('1; while (true) { break; }'), undefined);
assert.sameValue(eval('2; while (true) { 3; break; }'), 3);

assert.sameValue(
  eval('4; outer: do { while (true) { continue outer; } } while (false)'),
  undefined
);
assert.sameValue(
  eval('5; outer: do { while (true) { 6; continue outer; } } while (false)'), 6
);
