// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit can prevent evaluation of the right-side expressions
esid: sec-conditional-operator
info: |
    ConditionalExpression :
        ShortCircuitExpression
        ShortCircuitExpression ? AssignmentExpression : AssignmentExpression

    ShortCircuitExpression :
        LogicalORExpression
        CoalesceExpression

    CoalesceExpression :
        CoalesceExpressionHead ?? BitwiseORExpression

    CoalesceExpressionHead :
        CoalesceExpression
        BitwiseORExpression

    Runtime Semantics: Evaluation

    CoalesceExpression:CoalesceExpressionHead??BitwiseORExpression

    1. Let lref be the result of evaluating CoalesceExpressionHead.
    2. Let lval be ? GetValue(lref).
    3. If lval is undefined or null,
        a. Let rref be the result of evaluating BitwiseORExpression.
        b. Return ? GetValue(rref).
    4. Otherwise, return lval.
features: [coalesce-expression]
---*/

var x;
function poison() {
    throw new Test262Error('should not evaluate poison');
}

x = undefined;
x = undefined ?? 42 ?? undefined ?? poison();
assert.sameValue(x, 42);

x = undefined;
x = 42 ?? undefined ?? poison();
assert.sameValue(x, 42);

x = undefined;
x = undefined ?? 42 ?? poison();
assert.sameValue(x, 42);

x = undefined;
x = 42 ?? poison();
assert.sameValue(x, 42);
