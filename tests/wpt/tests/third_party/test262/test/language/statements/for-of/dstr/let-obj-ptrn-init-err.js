// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteration-statements
description: >
  Initializer is not allowed in head's ForDeclaration position.
info: |
  IterationStatement:
    for (ForDeclaration of AssignmentExpression) Statement
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

for (let {x} = 1 of []) {}
