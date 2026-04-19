// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-conditional-operator
es6id: 12.13
description: >
  The expression's LogicalORExpression sub-expression cannot include the `in`
  keyword in contexts where it is disallowed.
info: |
  Syntax

  ConditionalExpression[In, Yield] :
    LogicalORExpression[?In, ?Yield]
    LogicalORExpression[?In, ?Yield] ? AssignmentExpression[+In, ?Yield] : AssignmentExpression[?In, ?Yield]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

for ('' in {} ? 0 : 0; false; ) ;
