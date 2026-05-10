// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.7.5.11
description: >
    Completion value when head has no declaration and iteration is skipped
info: |
    IterationStatement : for ( LeftHandSideExpression in Expression ) Statement

    1. Let keyResult be ForIn/OfHeadEvaluation( « », Expression, enumerate).
    2. ReturnIfAbrupt(keyResult).

    13.7.5.12 Runtime Semantics: ForIn/OfHeadEvaluation

    [...]
    7. If iterationKind is enumerate, then
       a. If exprValue.[[value]] is null or undefined, then
          i. Return Completion{[[type]]: break, [[value]]: empty, [[target]]:
             empty}.
---*/

assert.sameValue(eval('var a; 1; for (a in undefined) { }'), undefined);
assert.sameValue(eval('var b; 2; for (b in undefined) { 3; }'), undefined);
assert.sameValue(eval('var c; 4; for (c in null) { }'), undefined);
assert.sameValue(eval('var d; 5; for (d in null) { 6; }'), undefined);
