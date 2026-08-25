// Copyright (C) 2016 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Rick Waldron
esid: sec-assignment-operators-runtime-semantics-evaluation
description: Compound Exponentiation Assignment Operator
info: |
    AssignmentExpression:
      LeftHandSideExpression AssignmentOperator AssignmentExpression

    1. Let lref be the result of evaluating LeftHandSideExpression.
    2. Let lval be ? GetValue(lref).
    3. Let rref be the result of evaluating AssignmentExpression.
    4. Let rval be ? GetValue(rref).
    5. Let op be the @ where AssignmentOperator is @=.
    6. Let r be the result of applying op to lval and rval as if evaluating the expression lval op rval.
    7. Perform ? PutValue(lref, r).
    8. Return r.
features: [exponentiation]
---*/

var base = -3;

assert.sameValue(base **= 3, -27, "(base **= 3) === -27; where base is -3");
