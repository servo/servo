// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.2.6
description: >
    Completion value when iteration completes due to expression value
info: |
    IterationStatement : do Statement while ( Expression ) ;

    1. Let V = undefined.
    2. Repeat
       a. Let stmt be the result of evaluating Statement.
       b. If LoopContinues(stmt, labelSet) is false, return
          Completion(UpdateEmpty(stmt, V)).
       c. If stmt.[[value]] is not empty, let V = stmt.[[value]].
       d. Let exprRef be the result of evaluating Expression.
       e. Let exprValue be GetValue(exprRef).
       f. ReturnIfAbrupt(exprValue).
       g. If ToBoolean(exprValue) is false, return NormalCompletion(V).
---*/

assert.sameValue(eval('1; do { } while (false)'), undefined);
assert.sameValue(eval('2; do { 3; } while (false)'), 3);
