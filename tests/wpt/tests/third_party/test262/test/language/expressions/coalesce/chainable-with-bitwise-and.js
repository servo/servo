// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    CoalesceExpression is chainable with the BitwiseANDExpression
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

x = null ?? 42 & 43;
assert.sameValue(x, 42, 'null ?? 42 & 43');

x = undefined ?? 42 & 43;
assert.sameValue(x, 42, 'null ?? 42 & 43');

x = false ?? 42 & 43;
assert.sameValue(x, false, 'false ?? 42 & 43');

x = true ?? 42 & 43;
assert.sameValue(x, true, 'true ?? 42 & 43');
