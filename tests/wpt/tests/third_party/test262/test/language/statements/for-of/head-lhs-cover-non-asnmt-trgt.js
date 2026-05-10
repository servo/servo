// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Head's LeftHandSideExpression must be a simple assignment target
info: |
    It is a Syntax Error if IsValidSimpleAssignmentTarget of
    LeftHandSideExpression is false.

    It is a Syntax Error if the LeftHandSideExpression is
    CoverParenthesizedExpressionAndArrowParameterList : ( Expression ) and
    Expression derives a production that would produce a Syntax Error according
    to these rules if that production is substituted for
    LeftHandSideExpression. This rule is recursively applied.
esid: sec-for-in-and-for-of-statements-static-semantics-early-errors
es6id: 13.7.5
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

for ((this) of []) {}
