// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: Logical Nullish Assignment Operator
features: [BigInt, logical-assignment-operators]
info: |
    AssignmentExpression:
      LeftHandSideExpression ??= AssignmentExpression

    1. Let lref be the result of evaluating LeftHandSideExpression.
    2. Let lval be ? GetValue(lref).
    3. If lval is neither undefined nor null, return lval.
    4. Let rref be the result of evaluating AssignmentExpression.
    5. Let rval be ? GetValue(rref).
    6. Perform ? PutValue(lref, rval).
    7. Return rval.

---*/

var value = 0n;
assert.sameValue(value ??= 1n, 0n, "(value ??= 1n) === 0n; where value = 0n");

value = 2n;
assert.sameValue(value ??= 1n, 2n, "(value ??= 1n) === 2n; where value = 2n");
