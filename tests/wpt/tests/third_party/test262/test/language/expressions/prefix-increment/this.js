// Copyright (C) 2023 Veera Sivarajan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-update-expressions-static-semantics-early-errors
description: >
  It is an early Syntax Error if AssignmentTargetType of UnaryExpression is not simple. (this)
info: |
  sec-static-semantics-assignmenttargettype

    PrimaryExpression: this 

    Return invalid.

  sec-update-expressions-static-semantics-early-errors

    UpdateExpression: ++ UnaryExpression 

    It is an early Syntax Error if AssignmentTargetType of UnaryExpression is not simple.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

++this;
