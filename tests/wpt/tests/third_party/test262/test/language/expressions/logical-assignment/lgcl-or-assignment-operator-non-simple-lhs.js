// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-static-semantics-early-errors
description: >
    It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is
    not simple.
negative:
  phase: parse
  type: SyntaxError
features: [logical-assignment-operators]

---*/

$DONOTEVALUATE();

function test() {}
test() ||= 1;
