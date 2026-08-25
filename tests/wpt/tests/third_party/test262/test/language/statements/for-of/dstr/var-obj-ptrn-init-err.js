// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteration-statements
description: >
  Initializer is not allowed in head's ForBinding position.
info: |
  IterationStatement:
    for (var ForBinding of AssignmentExpression) Statement
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

for (var {x} = 1 of []) {}
