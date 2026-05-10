// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Expression is allowed in head
info: |
    IterationStatement : for ( ForDeclaration in Expression ) Statement

    1. Let keyResult be the result of performing
       ForIn/OfHeadEvaluation(BoundNames of ForDeclaration, Expression,
       enumerate).
    2. ReturnIfAbrupt(keyResult).
    3. Return ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
       lexicalBinding, labelSet).
es6id: 13.7.5.11
---*/

var iterCount = 0;
var x;

for (x in null, { key: 0 }) {
  assert.sameValue(x, 'key');
  iterCount += 1;
}

assert.sameValue(iterCount, 1);
