// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invalid destructuring assignment pattern (object literal)
info: |
    It is a Syntax Error if LeftHandSideExpression is either an ObjectLiteral
    or an ArrayLiteral and if the lexical token sequence matched by
    LeftHandSideExpression cannot be parsed with no tokens left over using
    AssignmentPattern as the goal symbol.
esid: sec-for-in-and-for-of-statements-static-semantics-early-errors
es6id: 13.7.5
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

for ({ m() {} } of []) {}
