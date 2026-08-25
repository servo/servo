// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions
es6id: 14.4
description: A YieldExpression is not a valid LeftHandSideExpression
info: |
  AssignmentExpression[In, Yield] :
    ConditionalExpression[?In, ?Yield]
    [+Yield]YieldExpression[?In]
    ArrowFunction[?In, ?Yield]
    LeftHandSideExpression[?Yield] = AssignmentExpression[?In, ?Yield]
    LeftHandSideExpression[?Yield] AssignmentOperator AssignmentExpression[?In, ?Yield]

  LeftHandSideExpression[Yield] :
    NewExpression[?Yield]
    CallExpression[?Yield]
features: [generators]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function* g() {
  yield = 1;
}
